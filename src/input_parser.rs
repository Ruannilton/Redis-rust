use std::collections::{HashMap, VecDeque};

use crate::redis_types::{Command, ValueContainer};

#[derive(PartialEq)]
enum TokenValue {
    String(String),
    Array(VecDeque<TokenValue>),
}

impl Into<ValueContainer> for TokenValue {
    fn into(self) -> ValueContainer {
        match self {
            TokenValue::String(s) => ValueContainer::String(s),
            TokenValue::Array(_) => {
                panic!("Conversion from TokenValue::Array to ValueContainer is not supported")
            }
        }
    }
}

pub fn desserialize(input: Vec<u8>) -> Result<Command, Box<dyn std::error::Error>> {
    let mut tokens = split_input(input);
    let input_value = extract_input_values(&mut tokens)?;
    let command = convert_tokens_to_command(input_value);
    Ok(command)
}

fn split_input(input: Vec<u8>) -> VecDeque<String> {
    let delimiter = b"\r\n";
    let mut result = Vec::new();
    let mut start = 0;

    while let Some(pos) = input[start..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
    {
        let end = start + pos;
        result.push(&input[start..end]);
        start = end + delimiter.len();
    }

    if start < input.len() {
        result.push(&input[start..]);
    }

    let comands = result
        .iter()
        .map(|&bytes| String::from_utf8(bytes.to_vec()).expect("Invalid UTF-8 sequence"))
        .collect();

    comands
}

fn extract_input_values(
    tokens: &mut VecDeque<String>,
) -> Result<TokenValue, Box<dyn std::error::Error>> {
    let first_token = tokens.pop_front().ok_or(Box::new(std::fmt::Error))?;
    let token_id = first_token
        .chars()
        .next()
        .ok_or(Box::new(std::fmt::Error))?;

    match token_id {
        '+' => Ok(parse_simple_string(first_token)),
        '$' => {
            let str_content = tokens.pop_front().ok_or(Box::new(std::fmt::Error))?;
            Ok(parse_bulk_string(first_token, str_content))
        }
        '*' => parse_array(first_token, tokens),
        _ => Err(Box::new(std::fmt::Error)),
    }
}

fn parse_simple_string(mut token: String) -> TokenValue {
    token.remove(0);
    TokenValue::String(token)
}

fn parse_bulk_string(_token: String, next_token: String) -> TokenValue {
    TokenValue::String(next_token)
}

fn parse_array(
    token: String,
    token_queue: &mut VecDeque<String>,
) -> Result<TokenValue, Box<dyn std::error::Error>> {
    let mut vec = VecDeque::new();

    let itens = std::str::from_utf8(&token.as_bytes()[1..])?.parse::<i32>()?;

    let mut i = 0;

    while i < itens {
        let tkv = extract_input_values(token_queue)?;
        vec.push_back(tkv);
        i += 1;
    }

    Ok(TokenValue::Array(vec))
}

fn convert_tokens_to_command(token: TokenValue) -> Command {
    match token {
        TokenValue::String(simple) => handle_simple_command(simple),
        TokenValue::Array(aggregate) => handle_agregate_command(aggregate),
    }
}

fn handle_simple_command(command: String) -> Command {
    let upper_cmd = command.to_uppercase();
    match upper_cmd.as_str() {
        "PING" => Command::Ping,
        _ => Command::Invalid,
    }
}

fn handle_agregate_command(mut values: VecDeque<TokenValue>) -> Command {
    if values.len() == 1 {
        if let Some(value) = values.pop_front() {
            return match value {
                TokenValue::String(s) => handle_simple_command(s),
                _ => Command::Invalid,
            };
        }
        return Command::Invalid;
    }

    if let Some(TokenValue::String(command)) = values.pop_front() {
        let upper_cmd = command.to_uppercase();
        match upper_cmd.as_str() {
            "ECHO" => handle_agregate_echo(&mut values),
            "GET" => handle_agregate_get(&mut values),
            "SET" => handle_agregate_set(&mut values),
            "CONFIG" => handler_agregate_config(&mut values),
            "KEYS" => handle_agregate_keys(&mut values),
            "TYPE" => handle_agregate_type(&mut values),
            "XADD" => handle_agregate_xadd(&mut values),
            _ => Command::Invalid,
        }
    } else {
        Command::Invalid
    }
}

fn handle_agregate_echo(values: &mut VecDeque<TokenValue>) -> Command {
    if let Some(TokenValue::String(arg)) = values.pop_front() {
        return Command::Echo(arg);
    }
    Command::Invalid
}

fn handle_agregate_set(values: &mut VecDeque<TokenValue>) -> Command {
    if let (Some(TokenValue::String(key)), Some(TokenValue::String(value))) =
        (values.pop_front(), values.pop_front())
    {
        let mut expires_at: Option<u128> = None;

        if value.len() > 0 {
            let options = get_set_options(values);

            for (op, arg) in options {
                match (op.as_str(), arg) {
                    ("PX", Some(exp)) => expires_at = exp.parse::<u128>().ok(),
                    ("EX", Some(exp)) => expires_at = exp.parse::<u128>().map(|x| x * 1000).ok(),
                    _ => {}
                }
            }
        }

        return Command::Set(key, ValueContainer::String(value), expires_at);
    }
    Command::Invalid
}

fn get_set_options(values: &mut VecDeque<TokenValue>) -> HashMap<String, Option<String>> {
    let mut options = HashMap::new();
    loop {
        if values.is_empty() {
            break;
        }

        if let Some(TokenValue::String(op)) = values.pop_front() {
            let op = op.to_uppercase();

            match op.as_str() {
                "PX" => {
                    if let Some(TokenValue::String(val)) = values.pop_front() {
                        _ = options.insert(op, Some(val));
                    }
                }
                "EX" => {
                    if let Some(TokenValue::String(val)) = values.pop_front() {
                        _ = options.insert(op, Some(val));
                    }
                }
                _ => {}
            }
        }
    }
    options
}

fn handle_agregate_get(values: &mut VecDeque<TokenValue>) -> Command {
    if let Some(TokenValue::String(arg)) = values.pop_front() {
        return Command::Get(arg);
    }
    Command::Invalid
}

fn handle_agregate_keys(values: &mut VecDeque<TokenValue>) -> Command {
    if let Some(TokenValue::String(arg)) = values.pop_front() {
        return Command::Keys(arg);
    }
    Command::Invalid
}

fn handler_agregate_config(values: &mut VecDeque<TokenValue>) -> Command {
    if let Some(TokenValue::String(arg)) = values.pop_front() {
        let arg = arg.to_uppercase();
        return match arg.as_str() {
            "GET" => {
                if let Some(TokenValue::String(cfg_name)) = values.pop_front() {
                    Command::ConfigGet(cfg_name)
                } else {
                    Command::Invalid
                }
            }
            _ => Command::Invalid,
        };
    }
    Command::Invalid
}

fn handle_agregate_type(values: &mut VecDeque<TokenValue>) -> Command {
    if let Some(TokenValue::String(arg)) = values.pop_front() {
        return Command::Type(arg);
    }
    Command::Invalid
}

fn handle_agregate_xadd(values: &mut VecDeque<TokenValue>) -> Command {
    if let (Some(TokenValue::String(stream_id)), Some(TokenValue::String(entry_id))) =
        (values.pop_front(), values.pop_front())
    {
        let mut fields = Vec::new();

        while let (Some(TokenValue::String(key)), Some(TokenValue::String(value))) =
            (values.pop_front(), values.pop_front())
        {
            fields.push((key, value));
        }

        return Command::XAdd(stream_id, entry_id, fields);
    }

    Command::Invalid
}
