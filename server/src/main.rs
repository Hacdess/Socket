use std::{
    env, mem, 
    fs::{self, File}, 
    io::{self, Read},
    net::{Shutdown, TcpListener, TcpStream}, 
    path::Path
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use common::{FileList, Packet, Chunk};

pub struct Config {
    pub ip: String,
    pub port: String,
}



impl Config {
    pub fn get() -> Self {
        Self {
            ip: env::var("IP").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".into()),
        }
    }
}

fn get_files(file_name_path: &Path) -> Result<FileList, io::Error> {
    let content = fs::read_to_string(file_name_path)?;

    Ok(content
        .lines() // Break to lines
        .filter_map(|line| {             // to deal with
            let parts: Vec<&str> = line.split_whitespace().collect(); // split between name & size
            if parts.len() != 2 { // Incorrect
                return None;
            }

            if parts.len() != 2 {
                return None;
            }
            
            let name = parts[0];
            // Kiểm tra nếu tên không chứa ký tự null
            if name.contains('\0') {
                return None;
            }
            
            let size = fs::metadata(format!("resources/{}", name)).unwrap().len();
    
        

            Some((name.into(), size))
        })
        .collect::<Vec<_>>()
        .into_boxed_slice())
}

fn handle_client(mut stream: TcpStream, order: u8, files: FileList) -> io::Result<()> {
    println!("Connected to client {order}");

    dbg!(&stream);

    files.send(&mut stream)?;

    loop {
        let mut buf = [0; mem::size_of::<usize>()];
        if stream.read_exact(&mut buf).is_err() {
            break; // Nếu không còn dữ liệu để đọc, thoát vòng lặp
        }

        let filename_len = usize::from_be_bytes(buf);

        let mut filename_buf = vec![0; filename_len];
        stream.read_exact(&mut filename_buf)?; // Đọc tên file từ stream
        let filename = String::from_utf8(filename_buf).unwrap();

        if filename.is_empty() {
            continue;
        }

        let file_info = files.iter().find(|file| *file.0 == filename).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Requested file not found")
        })?;

        let file_path = Path::new("resources/").join(&*file_info.0);
        let mut file = File::open(file_path)?;

        loop {
            let chunk = Chunk::read(&mut file)?;
            chunk.send(&mut stream)?;
            if chunk.end() {
                println!("Finished sending file [{}]", file_info.0);
                break;
            }
        }
    }

    println!("All done with client {order}\nClosed connection with client {order}\n");

    Ok(())
}

fn main() -> io::Result<()> {
    let file_name_path = Path::new("files.txt");

    let files = get_files(file_name_path)?;

    let config = Config::get();
    let address = format!("{}:{}", config.ip, config.port);

    let listener = TcpListener::bind(&address).expect("Failed to bind TCP listener.");
    println!("Server is listening on {}", address);

    let mut count: u8 = 1;

    for stream in listener.incoming() {
        // Kiểm tra trạng thái running trước khi xử lý kết nối
            
        let stream = stream.unwrap();
        let files = files.clone();
    
        handle_client(stream, count, files).expect("Can't handle client {count}");
    
        count += 1;
    }

    Ok(())
}