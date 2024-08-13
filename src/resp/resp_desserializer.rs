use super::resp_error::RespError;
use super::resp_token::RespToken;
use core::str;
use std::{iter::Peekable, slice::Iter};

const SIMPLE_STRING_ID: u8 = b'+';
const BULK_STRING_ID: u8 = b'$';
const ARRAY_ID: u8 = b'*';

const SIMPLE_STRING_HEADER: &[u8; 1] = b"+";
const DELIMITER: &[u8; 2] = b"\r\n";

pub fn desserialize(input: &[u8]) -> Result<Vec<RespToken>, RespError> {
    let mut it = input.iter().peekable();
    let mut output = Vec::new();

    while let Some(&&identifier) = it.peek() {
        let tk = match identifier {
            SIMPLE_STRING_ID => desserialize_simple_string(&mut it),
            BULK_STRING_ID => desserialize_bulk_string(&mut it),
            ARRAY_ID => desserialize_array(&mut it),
            _ => return Err(RespError::EmptyStream),
        }?;

        output.push(tk);
    }

    Ok(output)
}

fn get_next_token(it: &mut Peekable<Iter<u8>>) -> Result<RespToken, RespError> {
    if let Some(&&identifier) = it.peek() {
        return match identifier {
            SIMPLE_STRING_ID => desserialize_simple_string(it),
            BULK_STRING_ID => desserialize_bulk_string(it),

            _ => return Err(RespError::InvalidToken(identifier.to_string())),
        };
    }

    Err(RespError::EmptyStream)
}

fn desserialize_simple_string(it: &mut Peekable<Iter<u8>>) -> Result<RespToken, RespError> {
    skip_exact(it, SIMPLE_STRING_HEADER)?;
    let str_bytes = read_until(it, DELIMITER)?;
    let str = str::from_utf8(&str_bytes).map_err(|_| RespError::InvalidUtf8Bytes)?;
    Ok(RespToken::String(str.to_owned()))
}

fn desserialize_bulk_string(it: &mut Peekable<Iter<u8>>) -> Result<RespToken, RespError> {
    _ = it.next();
    let str_len = read_lenght(it)?;
    let str_bytes = read_n_bytes(it, str_len)?;
    _ = it.next();
    _ = it.next();
    let str = String::from_utf8(str_bytes).map_err(|_| RespError::InvalidUtf8Bytes)?;
    Ok(RespToken::String(str))
}

fn desserialize_array(it: &mut Peekable<Iter<u8>>) -> Result<RespToken, RespError> {
    _ = it.next();
    let arr_len = read_lenght(it)?;
    let mut values = Vec::new();

    for _ in 0..arr_len {
        let tk = get_next_token(it)?;
        values.push(tk);
    }

    let response = RespToken::Array(values);
    Ok(response)
}

fn skip_exact(it: &mut Peekable<Iter<u8>>, buffer: &[u8]) -> Result<(), RespError> {
    for &byte in buffer {
        match it.next() {
            Some(&next_byte) if next_byte == byte => continue,
            _ => return Err(RespError::EmptyStream),
        }
    }
    Ok(())
}

fn read_until(it: &mut Peekable<Iter<u8>>, buffer: &[u8]) -> Result<Vec<u8>, RespError> {
    let mut output = Vec::with_capacity(it.len());

    loop {
        if let Some(&b) = it.next() {
            output.push(b);
            if output.ends_with(buffer) {
                break;
            }
        } else {
            return Err(RespError::EmptyStream);
        }
    }

    let buffer_len = buffer.len();
    let output_len = output.len();
    output.truncate(output_len - buffer_len);
    Ok(output)
}

fn read_lenght(it: &mut Peekable<Iter<u8>>) -> Result<usize, RespError> {
    let bytes = read_until(it, DELIMITER)?;
    let str = str::from_utf8(&bytes).map_err(|_| RespError::InvalidUtf8Bytes)?;
    let sz = usize::from_str_radix(str, 10).map_err(|_| RespError::InvalidInteger)?;
    Ok(sz)
}

fn read_n_bytes(it: &mut Peekable<Iter<u8>>, len: usize) -> Result<Vec<u8>, RespError> {
    let mut buffer = Vec::with_capacity(len);
    for _ in 0..len {
        let byte = it.next().ok_or(RespError::EmptyStream)?;
        buffer.push(*byte);
    }
    Ok(buffer)
}
