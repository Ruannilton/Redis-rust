use std::fmt::format;

pub fn to_resp_string(input: String) -> String {
    format!("+{}\r\n", input)
}

pub fn to_resp_bulk(input: String) -> String {
    format!("${}\r\n{}\r\n", input.len(), input)
}

pub fn to_err_string(input: String) -> String {
    format!("-{}\r\n", input)
}

pub fn to_resp_array(inputs: Vec<String>) -> String {
    let mut result = format!("*{}\r\n", inputs.len());
    for input in inputs {
        result.push_str(&format!("${}\r\n{}\r\n", input.len(), input));
    }
    result
}

pub fn null_resp_string() -> String {
    String::from("$-1\r\n")
}
