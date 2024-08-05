use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
};

fn format_size(mut x: u64) -> String {
    let suffixes = ["B", "KB", "MB", "GB"];
    let mut current = 0;
    while current + 1 < suffixes.len() && x >= 1024 {
        x /= 1024;
        current += 1;
    }

    format!("{x}{}", suffixes[current])
}

fn main() -> std::io::Result<()> {
    let mut stream: TcpStream;

    // Find and connect to server
    loop {
        let mut ip = String::new();
        let mut port = String::new();

        // Nhận IP từ người dùng
        print!("Input IP address: ");
        io::stdout().flush()?;
        ip.clear(); // Xóa dữ liệu trước đó
        io::stdin().read_line(&mut ip).expect("Failed to read line");
        let ip = ip.trim().to_string();

        // Nhận port từ người dùng
        print!("Input port: ");
        io::stdout().flush()?;
        port.clear(); // Xóa dữ liệu trước đó
        io::stdin().read_line(&mut port).expect("Failed to read line");
        let port = port.trim().to_string();

        let mut address = format!("{}:{}", ip, port);

        // Thử kết nối đến server
        if let Ok(s) = TcpStream::connect(&address) {
            stream = s;
            println!("Connected to the server at {}", address);
            break; // Kết nối thành công, thoát vòng lặp
        } else {
            println!("Couldn't connect to the server at {}. Please try again.", address);
        }
    }

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer)?;

    // Chuyển đổi dữ liệu từ bytes thành chuỗi
    let file_list = String::from_utf8_lossy(&buffer);

    // In danh sách file
    println!("Received file list from server:");
    println!("{}", file_list);

    stream.shutdown(Shutdown::Both).expect("Shutdown failed");
    Ok(())
}
