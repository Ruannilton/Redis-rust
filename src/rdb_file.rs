use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::{self, File},
    io::Read,
};

#[derive(Debug)]
struct DecodeError;

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decoding error")
    }
}

impl Error for DecodeError {}

#[derive(Debug)]
struct DecodeSizeError;

impl fmt::Display for DecodeSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decoding Size error")
    }
}

impl Error for DecodeSizeError {}

#[repr(u8)]
enum ValueTypeEncoding {
    StringEncoding = 0x00,
}

enum IntegerStringLen {
    SINGLEWORD = 0,
    DOUBLEWORD = 1,
    QUADWORD = 2,
}

#[repr(u8)]
enum OpCodes {
    EOF = 0xFF,
    SELECTDB = 0xFE,
    EXPIRETIME = 0xFD,
    EXPIRETIMEMS = 0xFC,
    RESIZEDB = 0xFB,
    METADATA = 0xFA,
}

impl TryFrom<u8> for OpCodes {
    type Error = OpCodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCodes::EOF as u8 => Ok(OpCodes::EOF),
            x if x == OpCodes::SELECTDB as u8 => Ok(OpCodes::SELECTDB),
            x if x == OpCodes::EXPIRETIME as u8 => Ok(OpCodes::EXPIRETIME),
            x if x == OpCodes::EXPIRETIMEMS as u8 => Ok(OpCodes::EXPIRETIMEMS),
            x if x == OpCodes::RESIZEDB as u8 => Ok(OpCodes::RESIZEDB),
            _ => Err(OpCodeError::InvalidOpCode),
        }
    }
}

#[derive(Debug)]
enum OpCodeError {
    InvalidOpCode,
}

impl fmt::Display for OpCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid OpCode")
    }
}

impl Error for OpCodeError {}

enum SizeEncodedValue {
    Size(usize),
    IntegerString(IntegerStringLen),
    LZFString(usize, usize),
}

#[derive(Debug)]
pub struct RdbFile {
    pub metadata: HashMap<String, String>,
    pub memory: HashMap<String, (String, Option<u128>)>,
}

impl RdbFile {
    pub fn open(path: String) -> Result<Self, Box<dyn Error>> {
        let mut file = fs::File::open(path)?;
        let mut instance = RdbFile {
            memory: HashMap::new(),
            metadata: HashMap::new(),
        };
        instance.read_content(&mut file)?;
        println!("Loaded: {:?}", instance);
        Ok(instance)
    }

    fn read_content(&mut self, file: &mut File) -> Result<(), Box<dyn Error>> {
        let magic_valid = Self::check_header_section(file)?;

        if !magic_valid {
            return Ok(());
        }

        self.read_metadata_section(file)?;
        self.read_database_section(file)?;
        Ok(())
    }

    fn check_header_section(stream: &mut File) -> Result<bool, Box<dyn Error>> {
        println!("Reading header");
        let mut buffer = [0u8; 5];
        let mut v_buffer = [0u8; 4];
        stream.read_exact(&mut buffer)?;
        stream.read_exact(&mut v_buffer)?;
        let match_header = "REDIS";
        let str_header = String::from_utf8(buffer.into())?;
        let str_version = String::from_utf8(v_buffer.into())?;

        println!("Header read: {}{}", str_header, str_version);
        let valid = str_header == match_header;
        if !valid {
            println!("Header invalid");
        }
        return Ok(valid);
    }

    fn read_metadata_section(&mut self, stream: &mut File) -> Result<(), Box<dyn Error>> {
        println!("Reading metadata");
        loop {
            let mut metadata_header = [0u8; 1];
            stream.read_exact(&mut metadata_header)?;

            if metadata_header[0] != OpCodes::METADATA as u8 {
                break;
            }

            let key = Self::read_string(stream)?;
            let value = Self::read_string(stream)?;
            println!("{:?}:{:?}", key, value);
            self.metadata.insert(key, value);
        }

        println!("Metadata read");
        Ok(())
    }

    fn read_database_section(&mut self, stream: &mut File) -> Result<(), Box<dyn Error>> {
        println!("Reading key value pairs");
        let db_index = Self::decode_size(stream)?;

        if let SizeEncodedValue::Size(db_index) = db_index {
            println!("Db Selected: {}", db_index);
        } else {
            println!("Failed to read db index");
            return Err(Box::new(DecodeError));
        }

        Self::read_database_table_size(stream)?;

        loop {
            let mut data_type_buffer = [0u8; 1];
            let mut expiration: Option<u128> = None;
            stream.read_exact(&mut data_type_buffer)?;

            if data_type_buffer[0] == OpCodes::EXPIRETIMEMS as u8 {
                let mut expiration_ms_buffer = [0u8; 8];
                stream.read_exact(&mut expiration_ms_buffer)?;

                let exp = u64::from_be_bytes(expiration_ms_buffer) as u128;
                expiration = Some(exp);
                stream.read_exact(&mut data_type_buffer)?;
            } else if data_type_buffer[0] == OpCodes::EXPIRETIME as u8 {
                let mut expiration_buffer = [0u8; 4];
                stream.read_exact(&mut expiration_buffer)?;

                let exp = (u32::from_be_bytes(expiration_buffer) as u128) * 1000;
                expiration = Some(exp);
                stream.read_exact(&mut data_type_buffer)?;
            }

            if data_type_buffer[0] == ValueTypeEncoding::StringEncoding as u8 {
                let key = Self::read_string(stream)?;
                let value = Self::read_string(stream)?;
                _ = self.memory.insert(key, (value, expiration));
            } else {
                break;
            }
        }
        println!("Key value pairs read");
        Ok(())
    }

    fn read_string(stream: &mut File) -> Result<String, Box<dyn Error>> {
        let decoded_size = Self::decode_size(stream)?;

        match decoded_size {
            SizeEncodedValue::Size(len) => Self::read_string_raw(stream, len),
            SizeEncodedValue::IntegerString(byte_len) => {
                return Self::read_integer_string(stream, byte_len);
            }
            SizeEncodedValue::LZFString(comp, ucomp) => Self::read_lzf_string(stream, comp, ucomp),
        }
    }

    fn read_string_raw(stream: &mut File, str_len: usize) -> Result<String, Box<dyn Error>> {
        let mut buffer = vec![0; str_len];
        stream.read_exact(&mut buffer)?;
        let res = String::from_utf8(buffer)?;
        Ok(res)
    }
    fn read_integer_string(
        stream: &mut File,
        byte_len: IntegerStringLen,
    ) -> Result<String, Box<dyn Error>> {
        return match byte_len {
            IntegerStringLen::SINGLEWORD => {
                let mut buffer = [0u8; 1];
                stream.read_exact(&mut buffer)?;
                let value = i8::from_be_bytes(buffer);
                let value_str = value.to_string();
                Ok(value_str)
            }
            IntegerStringLen::DOUBLEWORD => {
                let mut buffer = [0u8; 2];
                stream.read_exact(&mut buffer)?;
                let value = i16::from_be_bytes(buffer);
                let value_str = value.to_string();
                Ok(value_str)
            }
            IntegerStringLen::QUADWORD => {
                let mut buffer = [0u8; 4];
                stream.read_exact(&mut buffer)?;
                let value = i32::from_be_bytes(buffer);
                let value_str = value.to_string();
                Ok(value_str)
            }
        };
    }
    fn read_lzf_string(
        _stream: &mut File,
        _compressed_len: usize,
        _uncompressed_len: usize,
    ) -> Result<String, Box<dyn Error>> {
        Err(Box::new(DecodeError))
    }

    fn decode_size<R: Read>(stream: &mut R) -> Result<SizeEncodedValue, Box<dyn Error>> {
        let mut buffer = [0u8; 1];
        stream.read_exact(&mut buffer)?;

        let size_byte = buffer[0];
        let mode = size_byte >> 6;

        match mode {
            0 => {
                let size = size_byte & 0b00111111;
                return Ok(SizeEncodedValue::Size(size as usize));
            }
            1 => {
                let size1 = size_byte & 0b00111111;

                let mut buffer2 = [0u8; 1];
                stream.read_exact(&mut buffer2)?;

                let size2 = buffer2[0];
                let final_size: u16 = (((size1 & 0b00111111) as u16) << 8) | size2 as u16;
                return Ok(SizeEncodedValue::Size(final_size as usize));
            }
            2 => {
                let mut size = [0u8; 4];
                stream.read_exact(&mut size)?;
                let final_size = u32::from_be_bytes(size) as usize;
                Ok(SizeEncodedValue::Size(final_size))
            }
            3 => {
                let remaining = size_byte & 0b00111111;
                match remaining {
                    0 => Ok(SizeEncodedValue::IntegerString(
                        IntegerStringLen::SINGLEWORD,
                    )),
                    1 => Ok(SizeEncodedValue::IntegerString(
                        IntegerStringLen::DOUBLEWORD,
                    )),
                    2 => Ok(SizeEncodedValue::IntegerString(IntegerStringLen::QUADWORD)),
                    3 => Ok(SizeEncodedValue::LZFString(0, 0)),
                    _ => Err(Box::new(DecodeSizeError)),
                }
            }
            _ => Err(Box::new(DecodeSizeError)),
        }
    }

    fn read_database_table_size(stream: &mut File) -> Result<(), Box<dyn Error>> {
        let mut hash_table_header_buffer = [0u8; 1];
        stream.read_exact(&mut hash_table_header_buffer)?;
        Ok(if hash_table_header_buffer[0] == OpCodes::RESIZEDB as u8 {
            if let SizeEncodedValue::Size(key_table_size) = Self::decode_size(stream)? {
                println!("Key Value table size: {}", key_table_size);
            }
            if let SizeEncodedValue::Size(exp_table_size) = Self::decode_size(stream)? {
                println!("Expires table size: {}", exp_table_size);
            }
        } else {
            println!("Table size not read");
        })
    }
}
