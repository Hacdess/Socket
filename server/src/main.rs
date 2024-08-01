use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::{fs, io};
use std::path::Path;

fn handle_connection(mut stream: TcpStream, files: &Vec<&str>) -> io::Result<()> {
    stream.write_all(files.join("\n").as_bytes())?;

    Ok(())
}

fn main() -> io::Result<()>{
    // binding
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // get list of files and store it to string
    let path = Path::new("../files.txt");
    let reader = fs::read_to_string(path)?;
    let files: Vec<_> = reader.lines().filter(|line| !line.is_empty()).collect();

    for file in &files {
        if let Some((name, size)) = file.split_once(|c: char| c == ' ') {
            println!("File name: {}, File size: {}", name, size);
        } else {
            println!("Invalid syntax: {}", file);
        }
    }

    println!("Server is waiting...");
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Connection established!");
        handle_connection(stream, &files);
    }

    Ok(())
}