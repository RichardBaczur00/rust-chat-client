pub fn encode_message(msg: &String) -> String {
    let mut result: String = "".to_string();

    for chr in msg.chars() {
        result.push((chr as u8 ^ 0b00110101) as char);
    }

    return result;
}