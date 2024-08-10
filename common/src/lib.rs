use std::{io::{self, Read, Write}, mem, str};

pub trait Packet {
    fn send<T: Write>(&self, stream: &mut T) -> io::Result<()>;
    fn receive<T: Read>(stream: &mut T) -> io::Result<Self> where Self: Sized;
}

pub type FileList = Box<[(Box<str>, u64)]>;

impl Packet for FileList {
    fn send<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        // Send the number of files to stream
        stream.write_all(&(self.len() as usize).to_be_bytes())?;

        // Send the sizes of files in the list, each stored as u64 so transfer to bytes
        for (_, size) in self.iter() {
            stream.write_all(&size.to_be_bytes())?;
        }

        // Convert all the files' names into a single Stirng, seperated by '\0'
        let names = self.iter()
            .fold(String::new(), |acc, (name, _)| acc + "\0" + name);

        if names.is_empty() {
            stream.write_all(&0_usize.to_be_bytes())
        } else {
            // Convert the string of file names into bytes for ready to send
            let bytes = &names.as_bytes()[1..]; // From 1 to avoid \0
            // Announce the converted string's size
            stream.write_all(&(bytes.len() as usize).to_be_bytes())?;
            // Send the string files' names to stream
            stream.write_all(&bytes)
        }
    }

    fn receive<T: Read>(stream: &mut T) -> io::Result<Self> {
        // Get number of files in the list
        let len = {
            let mut buf = [0; mem::size_of::<usize>()];
            stream.read_exact(&mut buf)?;
            usize::from_be_bytes(buf)
        };

        // Each of file size is stored as u64 => size to store the whole files' sizes list is len * size_of<u64>
        let mut buf = vec![0; len * mem::size_of::<u64>()];
        stream.read_exact(&mut buf)?; // Store the files' sizes list into buf as bytes
        // Convert it into files' sizes list as iterator of u64
        let filesizes = buf.chunks(mem::size_of::<u64>())
            .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap()));

        // Get the files' names list's size to store
        let names_size = {
            let mut buf = [0; mem::size_of::<usize>()];
            stream.read_exact(&mut buf)?;
            usize::from_be_bytes(buf)
        };

        let mut buf = vec![0; names_size];
        stream.read_exact(&mut buf)?; // Store the whole files' names list into buf as bytes
        
        // Convert buf into list of files' names as iterator of pointer to str
        let filenames = str::from_utf8(&buf).unwrap()
            .splitn(len, '\0').map(|name| name.into());

        // Merge each file name to its file size using zip and return
        Ok(filenames.zip(filesizes).collect())
    }
}

pub struct Chunk {
    pub len: usize,
    buf: [u8; 1024],
}

impl Chunk {
    pub fn end(&self) -> bool {
        self.len < 1024
    }

    pub fn read<T: Read>(file: &mut T) -> io::Result<Self> {
        let mut buf = [0; 1024];
        let len = file.read(&mut buf)?;
        Ok(Chunk {len, buf})
    }

    pub fn write<T: Write>(mut self, file: &mut T) -> io::Result<bool> {
        file.write_all(&mut self.buf[..self.len])?;
        Ok(self.end())
    }
}

impl Packet for Chunk {
    fn send<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        let header = if self.end() { (1 << 15) | self.len as u16 } else { 0 };
        stream.write_all(&header.to_be_bytes())?;
        stream.write_all(&self.buf[..self.len])
    }

    fn receive<T: Read>(stream: &mut T) -> io::Result<Self> {
        let header = {
            let mut buf = [0; mem::size_of::<u16>()];
            stream.read_exact(&mut buf)?;
            u16::from_be_bytes(buf)
        };
        
        let end = (header >> 15) != 0;
        let mut buf = [0; 1024];
        let len = if end {
            (header as usize) & 0x3ff
        } else {
            1024
        };

        stream.read_exact(&mut buf[..len])?;
        Ok(Chunk { len, buf })
    }
}

#[derive(Debug)]
pub struct DownloadableFile {
    pub done: bool,
    pub file: Box<str>
}