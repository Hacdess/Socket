use std::{
    fs::{self, File}, io::{self, Write}, net::TcpStream, sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    }
};

use common::{Chunk, DownloadableFile, FileList, Packet, DEFAULT_PORT};

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
    let reader = fs::read_to_string(input_path)?;

    let files: Vec<Box<str>> = reader
        .lines()
        .filter_map(|line| {
            let filename = line.split_whitespace().next();
            filename.map(|f| f.to_string().into_boxed_str())
        })
        .collect();

    let mut queeue: Vec<Box<str>> = Vec::new();

    for file in files.iter() {
        let file_found = downloadable_files.iter().find(|df| df.file == *file);

        if let Some(df) = file_found {
            if !df.done {
                queeue.push(file.clone());
            }
        }
    }

    Ok(queeue)
}

fn main() -> std::io::Result<()> {
    let input_path = "input.txt";
    let output_path = "output/";

    let stopping = Arc::new(AtomicBool::new(false));
    let s = stopping.clone();

    if let Err(err) = ctrlc::set_handler(move || s.store(true, Ordering::SeqCst)) {
        eprintln!("ERROR: failed to set ctrl-c handler: {err}");
    }

    let address = {
        let ip = read_typed("Input IP address: ");
        format!("{}:{}", ip, DEFAULT_PORT)
    };

    let mut stream: TcpStream = TcpStream::connect(&address)
                       .expect("Couldn't connect to the server...");

    let file_list = FileList::recv(&mut stream)?;

    println!("Connected to server at {address}\n");
    println!("Availabe files for downloading:");
    if file_list.is_empty() {
        println!("No available file for downloading!\nProgram exiting...");
        return Ok(());
    } else {
        for (filename, filesize) in file_list.iter() {
            println!("{} {}", filename, format_size(*filesize));
        }
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
        if stopping.load(Ordering::SeqCst) {
            break;
        }

        let mut queeue = update_queeue(input_path, &downloadable_files)?;
    
        while !queeue.is_empty() {
            if stopping.load(Ordering::SeqCst) {
                break;
            }

            stream.write_all(&[1])?;
            
            let filename = queeue.remove(0);
        
            if let Some(file) = downloadable_files
                .iter_mut()
                .find(|f| f.file.as_ref().trim() == filename.as_ref().trim() && !f.done
            ) {
                stream.write_all(&(filename.len() as usize).to_be_bytes())?;
                stream.write_all(filename.as_bytes())?;
        
                let output_file_path = format!("{}{}", output_path, filename);
                let mut output_file = File::create(output_file_path)?;
        
                let mut progress: usize = 0;
                let max_size = match file_list.iter().find(|file| file.0 == filename) {
                    Some(file) => file.1,
                    None => {
                        eprintln!("Couldn't get file in the list: {}", filename);
                        continue;
                    }
                };
    
                loop {
                    if stopping.load(Ordering::SeqCst) {
                        break;
                    }

                    let chunk = Chunk::recv(&mut stream)?;                    
                    progress += chunk.len;
                    if chunk.write(&mut output_file)? {
                        print!("\rDownloading {} ..... 100.00%\n", filename);
                        print!("                                                \r");
                        println!("Finshed downloading [{}]", filename);
                        file.done = true;
                        break;
                    }
                    print!("\rDownloading {} ..... {:.2}%", filename, progress as f32 * 100.0 / max_size as f32);
                    io::stdout().flush()?;
                    
                }                
            }
        }
    }

    stream.write_all(&[0])?;
    println!();
    println!("Program is exiting...");
    Ok(())
}
