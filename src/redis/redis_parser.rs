use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::resp::resp_token::RespToken;

use super::{
    redis_error::RedisError,
    types::{command_token::CommandToken, value_container::ValueContainer},
};

pub fn parse_token_int_command(
    it: &mut Peekable<Iter<RespToken>>,
) -> Result<Vec<CommandToken>, RedisError> {
    let mut commands = Vec::new();

    while let Some(token) = it.next() {
        let cmd = match token {
            RespToken::Array(arr) => handle_aggregate_command(arr)?,
            RespToken::String(cmd) => {
                let arr = vec![RespToken::String(cmd.to_owned())];
                handle_aggregate_command(&arr)?
            }
            _ => return Err(RedisError::UnexpectedToken),
        };

        commands.push(cmd);
    }

    return Ok(commands);
}

fn handle_aggregate_command(token: &Vec<RespToken>) -> Result<CommandToken, RedisError> {
    let mut it = token.iter();

    if let Some(RespToken::String(command)) = it.next() {
        let cmd = command.to_uppercase();
        match cmd.as_str() {
            "PING" => Ok(CommandToken::Ping),
            "ECHO" => build_echo_command(&mut it),
            "GET" => build_get_command(&mut it),
            "SET" => build_set_command(&mut it),
            "CONFIG" => build_config_command(&mut it),
            "KEYS" => build_keys_command(&mut it),
            "TYPE" => build_type_command(&mut it),
            "XADD" => build_xadd_command(&mut it),
            "XRANGE" => build_xrange_command(&mut it),
            "XREAD" => build_xread_command(&mut it),
            _ => return Err(RedisError::InvalidCommand(cmd)),
        }
    } else {
        Err(RedisError::NoTokenAvailable)
    }
}

fn build_xread_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    let mut block_time: Option<u64> = None;
    let mut stream_names = Vec::with_capacity(it.len() - 2);

    while let Some(RespToken::String(token)) = it.next() {
        if token.to_uppercase() == "BLOCK" {
            if let Some(RespToken::String(time)) = it.next() {
                block_time = u64::from_str_radix(time, 10).ok();
            }
        } else if token.to_uppercase() == "STREAMS" {
            while let Some(token) = it.next() {
                if let RespToken::String(arg) = token {
                    stream_names.push(arg.to_owned());
                } else {
                    return Err(RedisError::UnexpectedToken);
                }
            }
        }
    }

    let stream_ids = stream_names.split_off(stream_names.len() / 2);

    Ok(CommandToken::XRead(block_time, stream_names, stream_ids))
}

fn build_xrange_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let (
        Some(RespToken::String(stream_id)),
        Some(RespToken::String(start)),
        Some(RespToken::String(end)),
    ) = (it.next(), it.next(), it.next())
    {
        return Ok(CommandToken::XRange(
            stream_id.to_owned(),
            start.to_owned(),
            end.to_owned(),
        ));
    }
    Err(RedisError::NoTokenAvailable)
}

fn build_xadd_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let (Some(RespToken::String(stream_id)), Some(RespToken::String(entry_id))) =
        (it.next(), it.next())
    {
        let mut fields = Vec::new();

        while let (Some(RespToken::String(key)), Some(RespToken::String(value))) =
            (it.next(), it.next())
        {
            fields.push((key.to_owned(), value.to_owned()));
        }

        Ok(CommandToken::XAdd(
            stream_id.to_owned(),
            entry_id.to_owned(),
            fields,
        ))
    } else {
        Err(RedisError::NoTokenAvailable)
    }
}

fn build_type_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(CommandToken::Type(s.to_owned()))
    } else {
        Err(RedisError::InvalidArgument)
    }
}

fn build_keys_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(CommandToken::Keys(s.to_owned()))
    } else {
        Err(RedisError::InvalidArgument)
    }
}

fn build_config_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let Some(RespToken::String(command)) = it.next() {
        match command.to_uppercase().as_str() {
            "GET" => {
                if let Some(RespToken::String(key)) = it.next() {
                    Ok(CommandToken::ConfigGet(key.to_owned()))
                } else {
                    Err(RedisError::InvalidArgument)
                }
            }
            _ => Err(RedisError::InvalidArgument),
        }
    } else {
        Err(RedisError::InvalidArgument)
    }
}

// TODO: extract expiration time
fn build_set_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let (Some(RespToken::String(key)), Some(value)) = (it.next(), it.next()) {
        let mut peekable = it.peekable();
        let options =
            search_optional_args(&mut peekable, HashMap::from([("PX", true), ("EX", true)]));
        let mut expires_at: Option<u128> = None;

        for (arg_name, arg_val) in options {
            match (arg_name.as_str(), arg_val) {
                ("PX", Some(ValueContainer::String(exp))) => expires_at = exp.parse::<u128>().ok(),
                ("EX", Some(ValueContainer::String(exp))) => {
                    expires_at = exp.parse::<u128>().map(|x| x * 1000).ok()
                }
                _ => {}
            }
        }

        Ok(CommandToken::Set(key.to_owned(), value.into(), expires_at))
    } else {
        Err(RedisError::InvalidArgument)
    }
}

fn build_get_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(CommandToken::Get(s.to_owned()))
    } else {
        Err(RedisError::InvalidArgument)
    }
}

fn build_echo_command(it: &mut Iter<RespToken>) -> Result<CommandToken, RedisError> {
    if let Some(arg) = it.next() {
        return match arg {
            RespToken::String(s) => Ok(CommandToken::Echo(ValueContainer::String(s.to_owned()))),
            RespToken::Integer(i) => Ok(CommandToken::Echo(ValueContainer::Integer(i.to_owned()))),
            _ => Err(RedisError::InvalidArgument),
        };
    }

    Err(RedisError::InvalidArgument)
}

fn search_optional_args(
    it: &mut Peekable<&mut Iter<RespToken>>,
    valid_args: HashMap<&str, bool>,
) -> HashMap<String, Option<ValueContainer>> {
    let mut map = HashMap::new();

    while let Some(possible_arg) = it.peek() {
        if let RespToken::String(arg_name) = possible_arg {
            let name = arg_name.to_uppercase();
            if let Some(&has_value) = valid_args.get(name.as_str()) {
                if has_value {
                    _ = it.next();
                    if let Some(val) = it.next() {
                        let v: ValueContainer = val.into();
                        map.insert(name.to_owned(), Some(v));
                    } else {
                        break;
                    }
                } else {
                    map.insert(name.to_owned(), None);
                    _ = it.next();
                }
            }
        } else {
            break;
        }
    }

    map
}
