use std::{collections::HashMap, error::Error, iter::Peekable, slice::Iter};

use crate::{
    redis_types::{Command, ValueContainer},
    resp_desserializer_error::RespDesserializerError,
    resp_invalid_command_error::RespInvalidCommandError,
    resp_type::RespToken,
};

pub fn parse_token_int_command(
    it: &mut Peekable<Iter<RespToken>>,
) -> Result<Vec<Command>, Box<dyn std::error::Error>> {
    let mut commands = Vec::new();

    while let Some(token) = it.peek() {
        let cmd = match token {
            RespToken::Array(arr) => handle_aggregate_command(arr)?,
            RespToken::String(cmd) => handle_simple_command(cmd)?,
            _ => {
                return Err(Box::new(RespDesserializerError::new(
                    "Command deve iniciar com um array ou string",
                )))
            }
        };

        commands.push(cmd);
    }

    return Ok(commands);
}

fn handle_simple_command(token: &String) -> Result<Command, Box<dyn std::error::Error>> {
    let upper_cmd = token.to_uppercase();
    match upper_cmd.as_str() {
        "PING" => Ok(Command::Ping),
        _ => return Err(Box::new(RespInvalidCommandError::new(upper_cmd.as_str()))),
    }
}

fn handle_aggregate_command(token: &Vec<RespToken>) -> Result<Command, Box<dyn std::error::Error>> {
    let mut it = token.iter();

    if let Some(RespToken::String(command)) = it.next() {
        return match command.to_uppercase().as_str() {
            "ECHO" => build_echo_command(&mut it),
            "GET" => build_get_command(&mut it),
            "SET" => build_set_command(&mut it),
            "CONFIG" => build_config_command(&mut it),
            "KEYS" => build_keys_command(&mut it),
            "TYPE" => build_type_command(&mut it),
            "XADD" => build_xadd_command(&mut it),
            _ => Err(Box::new(RespInvalidCommandError::new("Invalid command"))),
        };
    }
    Err(Box::new(RespInvalidCommandError::new("No value found")))
}

fn build_xadd_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let (Some(RespToken::String(stream_id)), Some(RespToken::String(entry_id))) =
        (it.next(), it.next())
    {
        let mut fields = Vec::new();

        while let (Some(RespToken::String(key)), Some(RespToken::String(value))) =
            (it.next(), it.next())
        {
            fields.push((key.to_owned(), value.to_owned()));
        }

        Ok(Command::XAdd(
            stream_id.to_owned(),
            entry_id.to_owned(),
            fields,
        ))
    } else {
        Err(Box::new(RespInvalidCommandError::new("No value found")))
    }
}

fn build_type_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(Command::Type(s.to_owned()))
    } else {
        Err(Box::new(RespInvalidCommandError::new(
            "Invalid argument type",
        )))
    }
}

fn build_keys_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(Command::Keys(s.to_owned()))
    } else {
        Err(Box::new(RespInvalidCommandError::new(
            "Invalid argument type",
        )))
    }
}

fn build_config_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let Some(RespToken::String(command)) = it.next() {
        match command.to_uppercase().as_str() {
            "GET" => {
                if let Some(RespToken::String(key)) = it.next() {
                    Ok(Command::ConfigGet(key.to_owned()))
                } else {
                    Err(Box::new(RespInvalidCommandError::new("Invalid argument")))
                }
            }
            _ => Err(Box::new(RespInvalidCommandError::new(
                "Invalid config argument",
            ))),
        }
    } else {
        Err(Box::new(RespInvalidCommandError::new("Invalid argument")))
    }
}

// TODO: extract expiration time
fn build_set_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
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

        Ok(Command::Set(key.to_owned(), value.into(), expires_at))
    } else {
        Err(Box::new(RespInvalidCommandError::new(
            "Invalid argument type",
        )))
    }
}

fn build_get_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let Some(RespToken::String(s)) = it.next() {
        Ok(Command::Get(s.to_owned()))
    } else {
        Err(Box::new(RespInvalidCommandError::new(
            "Invalid argument type",
        )))
    }
}

fn build_echo_command(it: &mut Iter<RespToken>) -> Result<Command, Box<dyn Error>> {
    if let Some(arg) = it.next() {
        return match arg {
            RespToken::String(s) => Ok(Command::Echo(ValueContainer::String(s.to_owned()))),
            RespToken::Integer(i) => Ok(Command::Echo(ValueContainer::Integer(i.to_owned()))),
            _ => Err(Box::new(RespInvalidCommandError::new(
                "Invalid argument type",
            ))),
        };
    }

    Err(Box::new(RespInvalidCommandError::new("Invalid argument")))
}

fn search_optional_args(
    it: &mut Peekable<&mut Iter<RespToken>>,
    valid_args: HashMap<&str, bool>,
) -> HashMap<String, Option<ValueContainer>> {
    let mut map = HashMap::new();

    while let Some(possible_arg) = it.peek() {
        if let RespToken::String(arg_name) = possible_arg {
            if let Some(&has_value) = valid_args.get(arg_name.as_str()) {
                if has_value {
                    if let Some(val) = it.next() {
                        let v: ValueContainer = val.into();
                        map.insert(arg_name.to_owned(), Some(v));
                    } else {
                        break;
                    }
                } else {
                    map.insert(arg_name.to_owned(), None);
                }
            }
        } else {
            break;
        }
    }

    map
}
