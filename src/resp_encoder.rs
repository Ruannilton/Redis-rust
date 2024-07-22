pub fn resp_encode_bulk_string(value: String) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}
