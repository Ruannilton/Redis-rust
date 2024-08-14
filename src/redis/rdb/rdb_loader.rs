use std::{
    collections::HashMap,
    fs::{self},
};

use crate::redis::{
    redis_error::RedisError,
    types::{entry_value::EntryValue, value_container::ValueContainer},
};

use super::{
    file_extensions::FileExt,
    rdb_types::{IntegerStringLen, OpCodes, SizeEncodedValue},
};

pub fn load(dir: &String, filename: &String) -> Result<HashMap<String, EntryValue>, RedisError> {
    let path = build_path(dir, filename);

    let file = fs::File::open(path).map_err(|err| RedisError::IOError(err))?;

    read_file(file)
}

fn build_path(dir: &String, filename: &String) -> String {
    let mut path = dir.to_owned();
    path.push('/');
    path.push_str(filename);
    path
}

fn read_file(mut file: impl FileExt) -> Result<HashMap<String, EntryValue>, RedisError> {
    check_header(&mut file)?;
    _ = read_metadata(&mut file)?;
    read_database(&mut file)
}

fn read_database(file: &mut impl FileExt) -> Result<HashMap<String, EntryValue>, RedisError> {
    let mut entries = HashMap::new();

    _ = decode_size(file)?;
    _ = read_database_size(file)?;

    loop {
        let mut op_code: OpCodes = file.next_u8()?.try_into()?;

        let exp = match op_code {
            OpCodes::ExpireTime => Some(file.next_u32()? as u128),
            OpCodes::ExpireTimeMs => Some(file.next_u64()? as u128),
            _ => None,
        };

        if exp.is_some() {
            op_code = file.next_u8()?.try_into()?;
        }

        if op_code == OpCodes::StringValue {
            let k = read_string(file)?;
            let v = read_string(file)?;
            let entry = EntryValue {
                expires_at: exp,
                value: ValueContainer::String(v),
            };
            entries.insert(k, entry);
        } else {
            break;
        }
    }

    Ok(entries)
}

fn check_header(file: &mut impl FileExt) -> Result<(), RedisError> {
    let match_header = "REDIS";
    let file_header = file.next_string(5)?;
    _ = file.next_string(4)?; // get version string

    let valid = file_header == match_header;

    if !valid {
        Err(RedisError::RDBInvalidHeader)
    } else {
        Ok(())
    }
}

fn read_metadata(file: &mut impl FileExt) -> Result<HashMap<String, String>, RedisError> {
    let mut metadata = HashMap::new();

    while file.next_u8()? == OpCodes::Metadata as u8 {
        let key = read_string(file)?;
        let val = read_string(file)?;

        metadata.insert(key, val);
    }

    Ok(metadata)
}

fn read_string(file: &mut impl FileExt) -> Result<String, RedisError> {
    let decoded_size = decode_size(file)?;

    let str = match decoded_size {
        SizeEncodedValue::Size(len) => file.next_string(len)?,
        SizeEncodedValue::IntegerString(int_len) => match int_len {
            IntegerStringLen::SingleWord => file.next_i8()?.to_string(),
            IntegerStringLen::DoubleWord => file.next_i16()?.to_string(),
            IntegerStringLen::QuadWord => file.next_i32()?.to_string(),
        },
        SizeEncodedValue::LZFString => panic!("LZF String not implemented"),
    };
    Ok(str)
}

fn decode_size(file: &mut impl FileExt) -> Result<SizeEncodedValue, RedisError> {
    let size = file.next_u8()?;
    let size_mode = size >> 6;
    let remaining = size & 0b00111111;
    match size_mode {
        0 => Ok(SizeEncodedValue::Size(remaining.into())),
        1 => {
            let ext = file.next_u8()?;
            let str_size = (remaining as u16) << 8 | (ext as u16);
            Ok(SizeEncodedValue::Size(str_size.into()))
        }
        2 => {
            let str_size = file.next_u32()?;
            Ok(SizeEncodedValue::Size(str_size as usize))
        }
        3 => match remaining {
            0 => Ok(SizeEncodedValue::IntegerString(
                IntegerStringLen::SingleWord,
            )),
            1 => Ok(SizeEncodedValue::IntegerString(
                IntegerStringLen::DoubleWord,
            )),
            2 => Ok(SizeEncodedValue::IntegerString(IntegerStringLen::QuadWord)),
            3 => Ok(SizeEncodedValue::LZFString),
            _ => Err(RedisError::RDBDecodeSizeError(size, size_mode, remaining)),
        },
        _ => Err(RedisError::RDBDecodeSizeError(size, size_mode, remaining)),
    }
}

fn read_database_size(file: &mut impl FileExt) -> Result<(), RedisError> {
    let header = file.peek()?;

    if header == OpCodes::ResizeDb as u8 {
        _ = decode_size(file)?;
        _ = decode_size(file)?;
    }

    Ok(())
}
