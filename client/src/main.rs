use std::{
    fs::{self, File},
    io::{self, Write},
    net::{Shutdown, TcpStream}
};

use common::{Chunk, DownloadableFile, FileList, Packet};

fn read_typed(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

fn format_size(mut x: u64) -> String {
    let suffixes = ["B", "KB", "MB", "GB"];
    let mut current = 0;
    while current + 1 < suffixes.len() && x >= 1024 {
        x /= 1024;
        current += 1;
    }

    format!("{x}{}", suffixes[current])
}

fn update_queeue(
    input_path: &str,
    downloadable_files: &Box<[DownloadableFile]>,
) -> io::Result<Vec<Box<str>>> {
    // Đọc nội dung của file và xử lý lỗi nếu có
    let reader = fs::read_to_string(input_path)?;

    // Chuyển đổi các tên file thành Vec<Box<str>>
    let files: Vec<Box<str>> = reader
        .lines()
        .filter_map(|line| {
            let filename = line.split_whitespace().next(); // Lấy phần tử đầu tiên
            filename.map(|f| f.to_string().into_boxed_str()) // Chuyển đổi Option<&str> thành Option<Box<str>>
        })
        .collect();

    let mut queeue: Vec<Box<str>> = Vec::new(); // Khởi tạo Vec để chứa các tên file cần xử lý

    for file in files.iter() {
        // So sánh Box<str> với Box<str>
        let file_found = downloadable_files.iter().find(|df| df.file == *file);

        // Nếu tìm thấy file và done là false thì thêm vào queeue
        if let Some(df) = file_found {
            if !df.done {
                queeue.push(file.clone());
            }
        }
    }

    Ok(queeue) // Trả về kết quả dưới dạng Result
}

fn main() -> std::io::Result<()> {
    let input_path = "input.txt";
    let output_path = "output/";

    // Find and connect to server
    let mut stream = loop {
        let ip = read_typed("Input IP address: ");
        let port = read_typed("Input port: ");
        let address = format!("{}:{}", ip, port);

        // Thử kết nối tới server
        match TcpStream::connect(&address) {
            Ok(s) => {
                println!("Connected to the server at {}", address);
                break s; // Kết nối thành công, thoát vòng lặp và trả về TcpStream
            }
            Err(_) => println!("Couldn't connect to the server at {}. Please try again.", address),
        }
    };

    dbg!(&stream);

    let file_list = FileList::recieve(&mut stream)?;
    println!("Availabe files for downloading:");
    for (filename, filesize) in file_list.iter() {
        println!("{} {}", filename, format_size(*filesize));
    }
    println!();

    let mut downloadable_files: Box<[DownloadableFile]> = file_list
        .iter()
        .map(|file: &(Box<str>, u64)| DownloadableFile {
            done: false,
            file: file.0.clone(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    loop {
        let mut queeue = update_queeue(input_path, &downloadable_files)?;
        if queeue.is_empty() {
            break;
        }

        // Xử lý các file trong queeue nếu không trống
        while !queeue.is_empty() {
            let filename = queeue.remove(0);

            // Kiểm tra xem file đã tải chưa
            if let Some(file) = downloadable_files.iter_mut().find(|f| f.file.as_ref().trim() == filename.as_ref().trim() && !f.done) {
                stream.write_all(&(filename.len() as usize).to_be_bytes())?;
                stream.write_all(filename.as_bytes())?;

                // Tạo đường dẫn file đầu ra
                let output_file_path = format!("{}{}", output_path, filename);
                let mut output_file = File::create(output_file_path)?;

                let mut progress: usize = 0;
                let max_size = match file_list.iter().find(|file| file.0 == filename) {
                    Some(file) => file.1,
                    None => {
                        eprintln!("File not found in the file list: {}", filename);
                        continue; // Hoặc break, tùy thuộc vào ngữ cảnh
                    }
                };

                // Nhận dữ liệu từ server và ghi vào file
                loop {
                    let chunk = Chunk::recieve(&mut stream)?;
                    progress += chunk.len;
                    if chunk.write(&mut output_file)? {
                        println!("Finshed downloading [{}]", filename);
                        file.done = true;
                        break;
                    }
                    print!("\rDownloading {} ..... {}%. ", filename, progress * 100 / max_size as usize);
                }                
            }
        }
    }

    stream.shutdown(Shutdown::Both).expect("Shutdown failed");
    Ok(())
}
