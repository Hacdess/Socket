use std::net::{TcpListener, TcpStream};
use std::{fs, io};
use std::path::Path;

fn main() -> io::Result<()>{
    // binding
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // get list of files and store it to string
    let path = Path::new("../files.txt");
    let reader = fs::read_to_string(path)?;
    let files: Vec<&str> = reader.lines().filter(|line| line.is_empty()).collect();

    //Waiting for clients
    // In thông tin về các file
    for file in files {
        // Tách tên file và kích thước
        if let Some((name, size)) = file.split_once(' ') {
            println!("Tên file: {}, Kích thước: {}", name, size);
        } else {
            // Xử lý trường hợp dòng không chứa dấu cách
            println!("Dòng không hợp lệ: {}", file);
        }
    }

    Ok(())
}