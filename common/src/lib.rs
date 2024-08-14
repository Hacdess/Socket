use std::{io::{self, Read, Write}, mem, str};

pub const DEFAULT_PORT: &str = "3000";

pub trait Packet {
    fn send<T: Write>(&self, stream: &mut T) -> io::Result<()>;
    fn recv<T: Read>(stream: &mut T) -> io::Result<Self> where Self: Sized;
}

pub type FileList = Box<[(Box<str>, u64)]>;

impl Packet for FileList {
    fn send<T: Write>(&self, stream: &mut T) -> io::Result<()> {
        stream.write_all(&(self.len() as u32).to_be_bytes())?;
        for (_, size) in self.iter() {
            stream.write(&size.to_be_bytes())?;
        }

        let names = self.iter()
            .fold(String::new(), |a, (name, _)| a + "\0" + name);

        if names.is_empty() {
            stream.write_all(&0_u32.to_be_bytes())
        } else {
            let bytes = &names.as_bytes()[1..];
            stream.write_all(&(bytes.len() as u32).to_be_bytes())?;
            stream.write_all(&bytes)
        }
    }

    fn recv<T: Read>(stream: &mut T) -> io::Result<Self> {
        let len = {
            let mut buf = [0; mem::size_of::<u32>()];
            stream.read_exact(&mut buf)?;
            u32::from_be_bytes(buf)
        } as usize;

        let mut buf = vec![0; len * mem::size_of::<u64>()];
        stream.read_exact(&mut buf)?;
        let filesizes = buf.chunks(mem::size_of::<u64>())
            .map(|bytes| u64::from_be_bytes(bytes.try_into().unwrap()));

        let names_size = {
            let mut buf = [0; mem::size_of::<u32>()];
            stream.read_exact(&mut buf)?;
            u32::from_be_bytes(buf)
        } as usize;

        let mut buf = vec![0; names_size];
        stream.read_exact(&mut buf)?;

        let filenames = str::from_utf8(&buf).unwrap()
            .splitn(len, '\0').map(|name| name.into());

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

    fn recv<T: Read>(stream: &mut T) -> io::Result<Self> {
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