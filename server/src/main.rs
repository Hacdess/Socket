use std::{
    env, mem,
    fs::{self, File},
    io::{self, Read}, 
    net::{TcpListener, TcpStream},
    path::Path, thread,
};

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

fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim().to_uppercase();
    let (value_str, unit) = size_str.split_at(size_str.len() - 2);
    
    let value = value_str.parse::<u64>().ok()?;
    match unit {
        "B" => Some(value),
        "KB" => Some(value * 1024),   
        "MB" => Some(value * 1024 * 1024),
        "GB" => Some(value * 1024 * 1024 * 1024),
        _ => None,
    }
}

fn get_files(file_name_path: &Path) -> Result<FileList, io::Error> {
    let content = fs::read_to_string(file_name_path)?;

    Ok(content.lines() // Break to lines
        .filter_map(|line| {             // to deal with
            let parts: Vec<&str> = line.split_whitespace().collect(); // split between name & size
            if parts.len() != 2 { // Incorrect
                return None;
            }

            let name = parts[0];
            let size = parse_size(parts[1])?;
            
            // Kiểm tra nếu tên không chứa ký tự null
            if name.contains('\0') {
                return None;
            }

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
            println!("All done");
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

    Ok(())
}


fn main() -> io::Result<()> {
    let file_name_path = Path::new("files.txt");
    //let files_path = Path::new("resources");

    let files = get_files(file_name_path)?;

    let config = Config::get();
    let address = format!("{}:{}", config.ip, config.port);

    let listener = TcpListener::bind(&address)?;
    println!("Server is listening on {}", address);

    let mut count: u8 = 1;
    let mut active_connection: Option<TcpStream> = None;

    for stream in listener.incoming() {
        match stream {
            Ok(new_stream) => {
                if active_connection.is_none() {
                    dbg!(&active_connection);
                    // Nếu không có kết nối nào đang hoạt động, chấp nhận kết nối mới
                    active_connection = Some(new_stream);
                    let stream = active_connection.take().unwrap();
                    let files = files.clone();

                    thread::spawn(move || {
                        if let Err(e) = handle_client(stream, count, files) {
                            eprintln!("Failed to handle client {}: {}", count, e);
                        }
                    });

                    count += 1;
                // Nếu đã có kết nối hoạt động, từ chối kết nối mới
                } else {
                    let _ = new_stream.shutdown(std::net::Shutdown::Both);
                    eprintln!("Connection refused because another connection is active.");
                }
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }

        thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
