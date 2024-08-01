use std::io::prelude::*;
use std::net::{SocketAddr, TcpStream};

fn main() -> std::io::Result<()> {
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 8080)),
        SocketAddr::from(([127, 0, 0, 1], 8081)),
        SocketAddr::from(([127, 0, 0, 1], 7878)),
    ];

    let mut stream = TcpStream::connect(&addrs[..]).expect("Couldn't connect to the server...");
    println!("Connected to the server!");

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    // Chuyển đổi dữ liệu từ bytes thành chuỗi
    let file_list = String::from_utf8_lossy(&buffer);

    // In danh sách file
    println!("Received file list from server:");
    println!("{}", file_list);

    Ok(())
} // 