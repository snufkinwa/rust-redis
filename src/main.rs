use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;

use parser::Command;
use rdb_parser::{parse_rdb_file, RDBData}; 
mod parser;
mod rdb_parser;

struct Config {
    dir: String,
    dbfilename: String,
}

struct ValueWithExpiry {
    value: String,
    expiry: Option<tokio::time::Instant>,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.iter().position(|x| x == "--dir").and_then(|i| args.get(i + 1)).map_or("/tmp/redis-data", |v| v).to_string();
    let dbfilename = args.iter().position(|x| x == "--dbfilename").and_then(|i| args.get(i + 1)).map_or("dump.rdb", |v| v).to_string();

    let config = Arc::new(Config { dir, dbfilename });

    println!("Server listening on 127.0.0.1:6379");

    let rdb_path = Path::new(&config.dir).join(&config.dbfilename);
    println!("Attempting to load RDB file from: {:?}", rdb_path);
    
    let initial_data = match parse_rdb_file(&rdb_path) {
        Ok(Some(data)) => data,
        Ok(None) => RDBData { keys: Vec::new() },
        Err(e) => {
            eprintln!("Error parsing RDB file: {}", e);
            RDBData { keys: Vec::new() }
        }
    };

    println!("Initial keys loaded: {:?}", initial_data.keys);

    let rdb_data = Arc::new(Mutex::new(initial_data));

    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let store = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let store = Arc::clone(&store);
        let config = Arc::clone(&config);
        let rdb_data = Arc::clone(&rdb_data);

        match listener.accept().await {
            Ok((mut socket, _)) => {
                println!("Accepted new connection");

                tokio::spawn(async move {
                    let mut buffer = BytesMut::with_capacity(1024);

                    loop {
                        match socket.read_buf(&mut buffer).await {
                            Ok(0) => {
                                println!("Connection closed by client");
                                return;
                            }
                            Ok(_) => {
                                if let Some(command) = parser::parse_command(&mut buffer) {
                                    match command {
                                        Command::ConfigGet(param) => {
                                            let response = match param.as_str() {
                                                "dir" => format!("*2\r\n$3\r\ndir\r\n${}\r\n{}\r\n", config.dir.len(), config.dir),
                                                "dbfilename" => format!("*2\r\n$10\r\ndbfilename\r\n${}\r\n{}\r\n", config.dbfilename.len(), config.dbfilename),
                                                _ => "$-1\r\n".to_string(),
                                            };
                                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                eprintln!("Failed to write response: {}", e);
                                                return;
                                            }
                                        }
                                        Command::Ping => {
                                            if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                                                eprintln!("Failed to write response: {}", e);
                                                return;
                                            }
                                        }
                                        Command::Echo(message) => {
                                            let response = format!("${}\r\n{}\r\n", message.len(), message);
                                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                eprintln!("Failed to write response: {}", e);
                                                return;
                                            }
                                        }
                                        Command::Set(key, value, expiry_ms) => {
                                            let mut store = store.lock().await;
                                            let expiry = expiry_ms.map(|ms| tokio::time::Instant::now() + tokio::time::Duration::from_millis(ms));
                                            store.insert(key, ValueWithExpiry { value, expiry });
                                            if let Err(e) = socket.write_all(b"+OK\r\n").await {
                                                eprintln!("Failed to write response: {}", e);
                                                return;
                                            }
                                        }
                                        Command::Get(key) => {
                                            let mut store = store.lock().await;
                                            if let Some(value_with_expiry) = store.get(&key) {
                                                if let Some(expiry) = value_with_expiry.expiry {
                                                    if tokio::time::Instant::now() > expiry {
                                                        store.remove(&key);
                                                        if let Err(e) = socket.write_all(b"$-1\r\n").await {
                                                            eprintln!("Failed to write response: {}", e);
                                                            return;
                                                        }
                                                    } else {
                                                        let response = format!("${}\r\n{}\r\n", value_with_expiry.value.len(), value_with_expiry.value);
                                                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                            eprintln!("Failed to write response: {}", e);
                                                            return;
                                                        }
                                                    }
                                                } else {
                                                    let response = format!("${}\r\n{}\r\n", value_with_expiry.value.len(), value_with_expiry.value);
                                                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                        eprintln!("Failed to write response: {}", e);
                                                        return;
                                                    }
                                                }
                                            } else {
                                                if let Err(e) = socket.write_all(b"$-1\r\n").await {
                                                    eprintln!("Failed to write response: {}", e);
                                                    return;
                                                }
                                            }
                                        }
                                        Command::Keys => {
                                            let rdb_data = rdb_data.lock().await;
                                            let keys = &rdb_data.keys;
                                            
                                            let mut response = format!("*{}\r\n", keys.len());
                                            for key in keys {
                                                response.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
                                            }
                                            
                                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                                eprintln!("Failed to write response: {}", e);
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading from socket: {:?}", e);
                                return;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}