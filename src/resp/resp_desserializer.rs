use std::{iter::Peekable, slice::Iter};

use crate::types::value_container::ValueContainer;

const SIMPLE_STRING_ID: u8 = b'+';
const SIMPLE_ERROR_ID: u8 = b'-';
const INTEGER_ID: u8 = b':';
const BULKS_STRING_ID: u8 = b'$';
const ARRAY_ID: u8 = b'*';
const NULL_ID: u8 = b'_';
const BOOLEAN_ID: u8 = b'#';
const DOUBLE_ID: u8 = b',';
const BIG_NUMBER_ID: u8 = b'(';
const BULK_ERROR_ID: u8 = b'!';
const VERBATIM_STRING_ID: u8 = b'=';
const MAP_ID: u8 = b'%';
const ATTRIBUTE_ID: u8 = b'`';
const SET_ID: u8 = b'~';

#[derive(Clone, Debug)]
pub enum RespTk {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespTk>),
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(String),
    BulkError(String),
    VerbatimString(String, String),
    Map(Vec<(RespTk, RespTk)>),
    Attribute(Vec<(RespTk, RespTk)>),
    Set(Vec<RespTk>),
    Invalid,
}

impl Into<String> for &RespTk {
    fn into(self) -> String {
        let delimiter = "\r\n";
        match self {
            RespTk::SimpleString(content) => {
                format!("{}{}{}", SIMPLE_STRING_ID, content, delimiter)
            }
            RespTk::SimpleError(content) => format!("{}{}{}", SIMPLE_ERROR_ID, content, delimiter),
            RespTk::Integer(content) => format!("{}{}{}", INTEGER_ID, content, delimiter),
            RespTk::BulkString(content) => format!(
                "{}{}{}{}{}",
                BULKS_STRING_ID,
                content.len(),
                delimiter,
                content,
                delimiter
            ),
            RespTk::Array(content) => {
                let arr: Vec<String> = content.into_iter().map(|t| t.into()).collect();
                format!("{}{}{}", ARRAY_ID, arr.len(), arr.join(""))
            }
            RespTk::Null => "_\r\n".into(),
            RespTk::Boolean(value) => {
                let ch = match value {
                    true => 't',
                    false => 'f',
                };
                format!("{}{}{}", BOOLEAN_ID, ch, delimiter)
            }
            RespTk::Double(value) => format!("{}{}{}", DOUBLE_ID, value, delimiter),
            RespTk::BigNumber(value) => format!("{}{}{}", BIG_NUMBER_ID, value, delimiter),
            RespTk::BulkError(value) => format!(
                "{}{}{}{}{}",
                BULK_ERROR_ID,
                value.len(),
                delimiter,
                value,
                delimiter
            ),
            RespTk::VerbatimString(encoding, value) => format!(
                "{}{}{}{}:{}{}",
                VERBATIM_STRING_ID,
                encoding.len() + value.len() + 1,
                delimiter,
                encoding,
                value,
                delimiter
            ),
            RespTk::Map(content) => {
                let arr: Vec<String> = content
                    .into_iter()
                    .map(|(k, v)| {
                        let k: String = k.into();
                        let v: String = v.into();
                        format!("{}{}", k, v)
                    })
                    .collect();
                format!("{}{}{}{}", MAP_ID, content.len(), delimiter, arr.join(""))
            }
            RespTk::Attribute(content) => {
                let arr: Vec<String> = content
                    .into_iter()
                    .map(|(k, v)| {
                        let k: String = k.into();
                        let v: String = v.into();
                        format!("{}{}", k, v)
                    })
                    .collect();
                format!("{}{}{}{}", MAP_ID, content.len(), delimiter, arr.join(""))
            }
            RespTk::Set(content) => {
                let arr: Vec<String> = content.into_iter().map(|t| t.into()).collect();
                format!("{}{}{}", SET_ID, arr.len(), arr.join(""))
            }
            RespTk::Invalid => format!("{}{}{}", SIMPLE_ERROR_ID, "ERROR invalid token", delimiter),
        }
    }
}

impl RespTk {
    pub fn get_command_name(&self) -> &str {
        let cmd_name = match &self {
            RespTk::SimpleString(name) => name,
            RespTk::BulkString(name) => name,
            RespTk::Array(content) => match content.first() {
                Some(RespTk::SimpleString(name)) => name,
                Some(RespTk::BulkString(name)) => name,
                _ => "INVALID",
            },
            _ => "INVALID",
        };

        cmd_name
    }

    pub fn get_command_args(&self) -> impl Iterator<Item = &RespTk> {
        if let RespTk::Array(values) = &self {
            let args = values.iter().skip(1);
            return args;
        }
        [].iter().skip(1)
    }

    pub fn get_content_string(&self) -> Option<String> {
        match &self {
            RespTk::SimpleString(s) => Some(s.into()),
            RespTk::BulkString(s) => Some(s.into()),
            RespTk::BigNumber(s) => Some(s.into()),
            RespTk::Boolean(b) => Some(b.to_string()),
            RespTk::Integer(i) => Some(i.to_string()),
            RespTk::Double(d) => Some(d.to_string()),
            _ => None,
        }
    }

    pub fn get_value(&self) -> ValueContainer {
        match &self {
            RespTk::SimpleString(s) => ValueContainer::String(s.into()),
            RespTk::BulkString(s) => ValueContainer::String(s.into()),
            RespTk::Integer(i) => ValueContainer::Integer(*i),
            RespTk::Array(arr) => {
                let values = arr.iter().map(|i| i.get_value()).collect();
                ValueContainer::Array(values)
            }
            RespTk::Boolean(b) => ValueContainer::Boolean(*b),
            _ => ValueContainer::Null,
        }
    }
}

pub fn parse_resp_buffer(buffer: &[u8]) -> Option<RespTk> {
    println!("parsing buffer: {:?}", buffer);
    let mut it = buffer.iter().peekable();
    return next_token(&mut it);
}

fn next_token(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let id_op = buffer.peek();
    if let None = id_op {
        return None;
    }

    let id = **id_op.unwrap();

    match id {
        SIMPLE_STRING_ID => parse_simple_string(buffer),
        SIMPLE_ERROR_ID => parse_simple_error(buffer),
        INTEGER_ID => parse_integer(buffer),
        BULKS_STRING_ID => parse_bulk_string(buffer),
        ARRAY_ID => parse_array(buffer),
        NULL_ID => parse_null(buffer),
        BOOLEAN_ID => parse_boolean(buffer),
        DOUBLE_ID => parse_double(buffer),
        BIG_NUMBER_ID => parse_big_number(buffer),
        BULK_ERROR_ID => parse_bulk_error(buffer),
        VERBATIM_STRING_ID => parse_verbatim_string(buffer),
        MAP_ID => parse_map(buffer),
        ATTRIBUTE_ID => parse_attribute(buffer),
        SET_ID => parse_set(buffer),
        _ => Some(RespTk::Invalid),
    }
}

fn parse_simple_string(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let content = read_until_delimitier(buffer);
    Some(RespTk::SimpleString(content))
}

fn parse_simple_error(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let content = read_until_delimitier(buffer);
    Some(RespTk::SimpleError(content))
}

fn parse_integer(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let content = read_until_delimitier(buffer);

    match i64::from_str_radix(&content, 10) {
        Ok(number) => Some(RespTk::Integer(number)),
        Err(_) => None,
    }
}

fn parse_bulk_string(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let len_h = read_until_delimitier(buffer);
    let len = i64::from_str_radix(&len_h, 10).unwrap();
    if len == -1 {
        return Some(RespTk::Null);
    }
    let content = read_until_delimitier(buffer);
    Some(RespTk::BulkString(content))
}

fn parse_array(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let len_h = read_until_delimitier(buffer);
    let len = i64::from_str_radix(&len_h, 10).unwrap();
    if len == -1 {
        return Some(RespTk::Null);
    }
    let mut tks = Vec::with_capacity(len as usize);

    for _ in 0..len {
        if let Some(tk) = next_token(buffer) {
            tks.push(tk);
        }
    }

    Some(RespTk::Array(tks))
}

fn parse_null(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let _ = read_until_delimitier(buffer);
    Some(RespTk::Null)
}

fn parse_boolean(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let val = buffer.next().unwrap();
    let _ = buffer.next();
    let _ = buffer.next();

    match val {
        b'f' => Some(RespTk::Boolean(false)),
        b't' => Some(RespTk::Boolean(true)),
        _ => Some(RespTk::Invalid),
    }
}

fn parse_double(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let content = read_until_delimitier(buffer);

    match content.parse::<f64>() {
        Ok(number) => Some(RespTk::Double(number)),
        Err(_) => Some(RespTk::Invalid),
    }
}

fn parse_big_number(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let content = read_until_delimitier(buffer);
    Some(RespTk::BigNumber(content))
}

fn parse_bulk_error(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let _ = read_until_delimitier(buffer);

    let content = read_until_delimitier(buffer);
    Some(RespTk::BulkError(content))
}

fn parse_verbatim_string(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let _ = read_until_delimitier(buffer);

    let content = read_until_delimitier(buffer);

    let mut parts = content.split(':');
    let encoding = parts.nth(0).unwrap().to_owned();
    let data = parts.nth(1).unwrap().to_owned();
    Some(RespTk::VerbatimString(encoding, data))
}

fn parse_map(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let len_h = read_until_delimitier(buffer);
    let len = i64::from_str_radix(&len_h, 10).unwrap();

    let mut content = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let key = next_token(buffer).unwrap();
        let value = next_token(buffer).unwrap();
        content.push((key, value));
    }

    Some(RespTk::Map(content))
}

fn parse_attribute(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let len_h = read_until_delimitier(buffer);
    let len = i64::from_str_radix(&len_h, 10).unwrap();

    let mut content = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let key = next_token(buffer).unwrap();
        let value = next_token(buffer).unwrap();
        content.push((key, value));
    }

    Some(RespTk::Attribute(content))
}

fn parse_set(buffer: &mut Peekable<Iter<u8>>) -> Option<RespTk> {
    let _ = buffer.next();
    let len_h = read_until_delimitier(buffer);
    let len = i64::from_str_radix(&len_h, 10).unwrap();
    if len == -1 {
        return Some(RespTk::Null);
    }
    let mut tks = Vec::with_capacity(len as usize);

    for _ in 0..len {
        if let Some(tk) = next_token(buffer) {
            tks.push(tk);
        }
    }

    Some(RespTk::Set(tks))
}

fn read_until_delimitier(buffer: &mut Peekable<Iter<u8>>) -> String {
    let mut result = String::new();
    let delimiter = "\r\n";

    while let Some(&b) = buffer.peek() {
        result.push(*b as char);
        buffer.next();

        if result.ends_with(delimiter) {
            result.truncate(result.len() - delimiter.len());
            return result;
        }
    }

    String::new()
}
