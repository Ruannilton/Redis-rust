use std::collections::VecDeque;

#[derive(PartialEq)]
enum TokenValue {
    String(String),
    Array(Vec<TokenValue>),
    Null,
}

#[derive(Debug)]
pub enum RedisCommand {
    Ping,
    Echo(String),
    Invalid,
}

pub fn get_redis_command(buffer: &[u8]) -> RedisCommand {
    let mut bin_tokens: VecDeque<String> = split_buffer(buffer).into_iter().collect();
    parse_to_command(&mut bin_tokens)
}

fn split_buffer(buffer: &[u8]) -> Vec<String> {
    let delimiter = b"\r\n";
    let mut result = Vec::new();
    let mut start = 0;

    while let Some(pos) = buffer[start..]
        .windows(delimiter.len())
        .position(|window| window == delimiter)
    {
        let end = start + pos;
        result.push(&buffer[start..end]);
        start = end + delimiter.len();
    }

    if start < buffer.len() {
        result.push(&buffer[start..]);
    }

    let comands = result
        .iter()
        .map(|&bytes| String::from_utf8(bytes.to_vec()).expect("Invalid UTF-8 sequence"))
        .collect();

    comands
}

fn parse_to_command(token_queue: &mut VecDeque<String>) -> RedisCommand {
    let token_parsed = parse(token_queue).map_or(TokenValue::Null, |x| x);

    if token_parsed == TokenValue::Null {
        return RedisCommand::Invalid;
    };

    match token_parsed {
        TokenValue::String(cmd) => parse_string_to_redis_command(cmd),
        TokenValue::Array(cmds) => parse_vector_to_redis_command(cmds),
        TokenValue::Null => RedisCommand::Invalid,
    }
}

fn parse(token_queue: &mut VecDeque<String>) -> Option<TokenValue> {
    if token_queue.is_empty() {
        return None;
    }

    let tk = token_queue.pop_front().unwrap();
    let tk_identifier = tk.chars().next().unwrap();

    return match tk_identifier {
        '+' => Some(TokenValue::String(parse_simple_string(tk))),
        '$' => Some(TokenValue::String(parse_bulk_string(
            tk,
            token_queue.pop_front().unwrap(),
        ))),
        '*' => Some(TokenValue::Array(parse_array(tk, token_queue))),
        _ => None,
    };
}

fn parse_simple_string(mut token: String) -> String {
    token.remove(0);
    token
}

fn parse_bulk_string(_token: String, next_token: String) -> String {
    next_token
}

fn parse_array(token: String, token_queue: &mut VecDeque<String>) -> Vec<TokenValue> {
    let mut vec = Vec::new();

    let itens = std::str::from_utf8(&token.as_bytes()[1..])
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let mut i = 0;

    while i < itens {
        let tkv = parse(token_queue).unwrap();
        vec.push(tkv);
        i += 1;
    }

    vec
}

fn parse_string_to_redis_command(cmd: String) -> RedisCommand {
    let upper_cmd = cmd.to_uppercase();
    match upper_cmd.as_str() {
        "PING" => RedisCommand::Ping,
        _ => RedisCommand::Invalid,
    }
}

fn parse_vector_to_redis_command(cmds: Vec<TokenValue>) -> RedisCommand {
    let mut cmd_values: VecDeque<String> = cmds
        .into_iter()
        .map(|x| match x {
            TokenValue::String(s) => s,
            _ => String::new(),
        })
        .collect();

    let upper_cmd = cmd_values.pop_front().unwrap().to_uppercase();

    match upper_cmd.as_str() {
        "ECHO" => RedisCommand::Echo(cmd_values.pop_front().unwrap()),
        "PING" => RedisCommand::Ping,
        _ => RedisCommand::Invalid,
    }
}

pub fn encode_as_bulk_string(value: String) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}
