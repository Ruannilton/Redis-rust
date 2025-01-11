use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_time_ms() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}
pub fn base64_to_bin(encoded: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut decoded = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for byte in encoded.bytes() {
        if byte == b'=' {
            break;
        }

        let value = BASE64_CHARS
            .iter()
            .position(|&c| c == byte)
            .ok_or("Invalid base64 character")? as u32;
        buffer = (buffer << 6) | value;
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            decoded.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    Ok(decoded)
}
