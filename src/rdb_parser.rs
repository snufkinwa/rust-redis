use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use bytes::{Buf, BytesMut};

#[derive(Debug)]
pub struct RDBData {
    pub keys: Vec<String>,
}

pub fn parse_rdb_file(path: &Path) -> io::Result<Option<RDBData>> {
    if !path.exists() {
        println!("RDB file not found at {:?}", path);
        return Ok(None);
    }

    println!("Opening RDB file at {:?}", path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut bytes = BytesMut::from(&buffer[..]);
    let mut rdb_data = RDBData { keys: Vec::new() };

    if bytes.len() < 9 || &bytes[..9] != b"REDIS0011" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid RDB header"));
    }
    bytes.advance(9);

    while bytes.has_remaining() {
        let type_byte = bytes.get_u8();
        
        match type_byte {
            0xFF => break, 
            0xFA => {
                let _key = read_string(&mut bytes)?;
                let _value = read_string(&mut bytes)?;
            }
            0xFE => {
                let _db_number = read_length(&mut bytes)?;
                
                if bytes.has_remaining() && bytes[0] == 0xFB {
                    bytes.advance(1);
                    let _hash_table_size = read_length(&mut bytes)?;
                    let _expires_table_size = read_length(&mut bytes)?;
                }
            }
            0xFD => {
                bytes.advance(4);
                if bytes.has_remaining() {
                    let value_type = bytes.get_u8();
                    if value_type == 0 {
                        let key = read_string(&mut bytes)?;
                        let _value = read_string(&mut bytes)?;
                        rdb_data.keys.push(key);
                    }
                }
            }
            0xFC => {
                bytes.advance(8);
                if bytes.has_remaining() {
                    let value_type = bytes.get_u8();
                    if value_type == 0 {
                        let key = read_string(&mut bytes)?;
                        let _value = read_string(&mut bytes)?;
                        rdb_data.keys.push(key);
                    }
                }
            }
            0 => {
                let key = read_string(&mut bytes)?;
                let _value = read_string(&mut bytes)?;
                rdb_data.keys.push(key);
            }
            _ => continue,
        }
    }

    println!("Successfully loaded keys: {:?}", rdb_data.keys);
    Ok(Some(rdb_data))
}

fn read_string(bytes: &mut BytesMut) -> io::Result<String> {
    let length = read_length(bytes)?;
    if bytes.remaining() < length {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Incomplete string data"));
    }
    let string_data = bytes.split_to(length).to_vec();
    String::from_utf8(string_data)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in string"))
}

fn read_length(bytes: &mut BytesMut) -> io::Result<usize> {
    if !bytes.has_remaining() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF while reading length"));
    }

    let first = bytes.get_u8();
    let len = match first >> 6 {
        0 => {
            (first & 0x3F) as usize
        }
        1 => {
            if !bytes.has_remaining() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF reading length"));
            }
            let next = bytes.get_u8();
            (((first & 0x3F) as usize) << 8) | (next as usize)
        }
        2 => {
            if bytes.remaining() < 4 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF reading length"));
            }
            (bytes.get_u8() as usize) << 24 |
            (bytes.get_u8() as usize) << 16 |
            (bytes.get_u8() as usize) << 8 |
            (bytes.get_u8() as usize)
        }
        3 => {
            match first & 0x3F {
                0 => 1, 
                1 => 2, 
                2 => 4, 
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid special encoding")),
            }
        }
        _ => unreachable!(),
    };
    Ok(len)
}