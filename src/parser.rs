use bytes::{Buf, BytesMut};


#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>), 
    Get(String),
    ConfigGet(String),
    Keys,
}

pub fn parse_command(buffer: &mut BytesMut) -> Option<Command> {
    if buffer.starts_with(b"*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n") {
        buffer.advance(b"*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n".len());
        if buffer.starts_with(b"$") {
            if let Some(len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                let len_str = String::from_utf8_lossy(&buffer[1..len_end + 1]);
                let param_len: usize = len_str.parse().ok()?;
                buffer.advance(len_end + 3);

                if buffer.len() >= param_len + 2 {
                    let param = String::from_utf8_lossy(&buffer[..param_len]).to_string();
                    buffer.advance(param_len + 2);
                    return Some(Command::ConfigGet(param));
                }
            }
        }
    }
    if buffer.starts_with(b"*1\r\n$4\r\nPING\r\n") {
        buffer.advance(b"*1\r\n$4\r\nPING\r\n".len());
        return Some(Command::Ping);
    } 
    else if buffer.starts_with(b"*2\r\n$4\r\nECHO\r\n") {
        let echo_prefix_len = b"*2\r\n$4\r\nECHO\r\n".len();
        if buffer.len() > echo_prefix_len {
            buffer.advance(echo_prefix_len);

            if buffer.starts_with(b"$") {
                if let Some(len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                    let len_str = String::from_utf8_lossy(&buffer[1..len_end + 1]);
                    let arg_len: usize = len_str.parse().ok()?;

                    buffer.advance(len_end + 3);

                    if buffer.len() >= arg_len + 2 {
                        let argument = String::from_utf8_lossy(&buffer[..arg_len]).to_string();
                        buffer.advance(arg_len + 2);
                        return Some(Command::Echo(argument));
                    }
                }
            }
        }
    }
    else if buffer.starts_with(b"*3\r\n$3\r\nSET\r\n") || buffer.starts_with(b"*5\r\n$3\r\nSET\r\n") {
        let set_prefix_len = b"*3\r\n$3\r\nSET\r\n".len();
        buffer.advance(set_prefix_len);

        // Parse the key
        if buffer.starts_with(b"$") {
            if let Some(key_len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                let key_len_str = String::from_utf8_lossy(&buffer[1..key_len_end + 1]);
                let key_len: usize = key_len_str.parse().ok()?;
                buffer.advance(key_len_end + 3);

                if buffer.len() >= key_len + 2 {
                    let key = String::from_utf8_lossy(&buffer[..key_len]).to_string();
                    buffer.advance(key_len + 2);

                    // Parse the value
                    if buffer.starts_with(b"$") {
                        if let Some(value_len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                            let value_len_str = String::from_utf8_lossy(&buffer[1..value_len_end + 1]);
                            let value_len: usize = value_len_str.parse().ok()?;
                            buffer.advance(value_len_end + 3);

                            if buffer.len() >= value_len + 2 {
                                let value = String::from_utf8_lossy(&buffer[..value_len]).to_string();
                                buffer.advance(value_len + 2);

                                if buffer.starts_with(b"$2\r\nPX\r\n") || buffer.starts_with(b"$2\r\npx\r\n") {
                                    buffer.advance(b"$2\r\nPX\r\n".len());
                                    if buffer.starts_with(b"$") {
                                        if let Some(px_len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                                            let px_len_str = String::from_utf8_lossy(&buffer[1..px_len_end + 1]);
                                            let px_len: usize = px_len_str.parse().ok()?;
                                            buffer.advance(px_len_end + 3);

                                            if buffer.len() >= px_len + 2 {
                                                let px_value = String::from_utf8_lossy(&buffer[..px_len]).to_string();
                                                buffer.advance(px_len + 2);
                                                if let Ok(px_value_ms) = px_value.parse::<u64>() {
                                                    return Some(Command::Set(key, value, Some(px_value_ms)));
                                                }
                                            }
                                        }
                                    }
                                }
                                return Some(Command::Set(key, value, None));
                            }
                        }
                    }
                }
            }
        }
    }
    else if buffer.starts_with(b"*2\r\n$3\r\nGET\r\n") {
        let get_prefix_len = b"*2\r\n$3\r\nGET\r\n".len();
        buffer.advance(get_prefix_len);

        if buffer.starts_with(b"$") {
            if let Some(key_len_end) = buffer[1..].iter().position(|&b| b == b'\r') {
                let key_len_str = String::from_utf8_lossy(&buffer[1..key_len_end + 1]);
                let key_len: usize = key_len_str.parse().ok()?;
                buffer.advance(key_len_end + 3);

                if buffer.len() >= key_len + 2 {
                    let key = String::from_utf8_lossy(&buffer[..key_len]).to_string();
                    buffer.advance(key_len + 2);
                    return Some(Command::Get(key));
                }
            }
        }
    } else if buffer.starts_with(b"*2\r\n$4\r\nKEYS\r\n") {
        buffer.advance(b"*2\r\n$4\r\nKEYS\r\n".len());
        return Some(Command::Keys);
    }
    None
}
