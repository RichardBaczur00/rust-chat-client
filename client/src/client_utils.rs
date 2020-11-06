struct Message {
    source: String,
    encryption_flag: u8,
    message: String
}


fn decode_message(buff: &Vec<u8>) -> Message {
    

    // Placeholders here until the function is implemented
    return Message {
        source: "".to_string(),
        encryption_flag: 0,
        message: "".to_string()
    }
}