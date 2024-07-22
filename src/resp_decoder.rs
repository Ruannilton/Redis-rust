use std::io::Read;
use std::str;

enum OpCode {
    Array(Vec<OpCode>),
    BulkString(String),
}

trait Decoder {
    fn decode(buffer: &[u8]) -> Option<(OpCode, usize)>;
}

fn read_u32(buffer: &[u8]) -> u32 {
    let num_str = str::from_utf8(buffer).unwrap();
    let num = num_str.parse::<u32>().unwrap();
    num
}

pub struct Command {
    pub command: String,
    pub args: Option<Vec<String>>,
}

pub struct BufferDecoder {}

impl BufferDecoder {
    pub fn decode(buffer: &[u8]) -> Option<Command> {
        let buffer_len = buffer.len();

        if buffer_len == 0 {
            return None;
        }

        if let Some((code, _)) = GenericDecoder::decode(buffer) {
            return match code {
                OpCode::Array(array) => {
                    if array.is_empty() {
                        return None;
                    }

                    let mut iter = array.into_iter();

                    match iter.next() {
                        Some(OpCode::BulkString(command)) => {
                            let args: Vec<String> = iter
                                .filter_map(|op| {
                                    if let OpCode::BulkString(arg) = op {
                                        Some(arg)
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            Some(Command {
                                command: command.to_uppercase(),
                                args: if args.is_empty() { None } else { Some(args) },
                            })
                        }
                        _ => None,
                    }
                }

                OpCode::BulkString(str) => Some(Command {
                    command: str.to_uppercase(),
                    args: None,
                }),
            };
        }
        None
    }
}

struct GenericDecoder {}
impl Decoder for GenericDecoder {
    fn decode(buffer: &[u8]) -> Option<(OpCode, usize)> {
        match buffer[0] as char {
            '$' => BulkStringDecoder::decode(buffer),
            '*' => ArrayDecoder::decode(buffer),
            _ => None,
        }
    }
}

struct BulkStringDecoder {}
impl Decoder for BulkStringDecoder {
    fn decode(buffer: &[u8]) -> Option<(OpCode, usize)> {
        let string_lenght = read_u32(&buffer[1..]);
        let skip = string_lenght + 4;

        let mut buff = String::new();
        let from = skip as usize;
        let to = from + (string_lenght as usize) + 1;
        let str_slice = (&buffer[from..to]).read_to_string(&mut buff);

        match str_slice {
            Ok(_) => {
                let readed_bytes = (skip + string_lenght + 2) as usize;
                Some((OpCode::BulkString(buff), readed_bytes))
            }
            _ => None,
        }
    }
}

struct ArrayDecoder {}
impl Decoder for ArrayDecoder {
    fn decode(buffer: &[u8]) -> Option<(OpCode, usize)> {
        let element_count = read_u32(&buffer[1..]);
        let skip = (element_count + 4) as usize;

        let mut op_codes = Vec::<OpCode>::new();
        op_codes.reserve(element_count as usize);

        let mut start = skip;
        let mut working_buffer = &buffer[start..];
        let mut readed_bytes: usize = 0;

        loop {
            if let Some((code, readed)) = GenericDecoder::decode(working_buffer) {
                op_codes.push(code);
                readed_bytes += readed;
                start += readed + 1;

                if start >= buffer.len() {
                    break;
                }

                working_buffer = &buffer[start..];
            } else {
                break;
            }
        }

        return Some((OpCode::Array(op_codes), readed_bytes));
    }
}
