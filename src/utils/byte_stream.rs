use std::io::{Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct ByteStream<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> ByteStream<'a> {
    pub fn new(stream: &'a [u8]) -> Self {
        ByteStream {
            cursor: Cursor::new(stream),
        }
    }

    pub fn read_u8(&mut self) -> std::io::Result<u8> {
        self.cursor.read_u8()
    }

    pub fn read_u16(&mut self) -> std::io::Result<u16> {
        self.cursor.read_u16::<LittleEndian>()
    }

    pub fn read_u32(&mut self) -> std::io::Result<u32> {
        self.cursor.read_u32::<LittleEndian>()
    }

    pub fn read_i16(&mut self) -> std::io::Result<i16> {
        self.cursor.read_i16::<LittleEndian>()
    }

    pub fn read_i32(&mut self) -> std::io::Result<i32> {
        self.cursor.read_i32::<LittleEndian>()
    }

    pub fn read_f32(&mut self) -> std::io::Result<f32> {
        self.cursor.read_f32::<LittleEndian>()
    }

    pub fn read_string(&mut self, size: usize) -> std::io::Result<String> {
        let mut buf = vec![0; size];
        self.cursor.read_exact(&mut buf)?;
        let pos = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Ok(String::from_utf8_lossy(&buf[..pos]).to_string())
    }

    pub fn read(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; size];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn get_size(&self) -> u64 {
        self.cursor.get_ref().len() as u64
    }

    pub fn get_position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn set_position(&mut self, position: u64) {
        self.cursor.set_position(position);
    }

    pub fn skip(&mut self, size: u64) -> std::io::Result<u64> {
        self.cursor.seek(SeekFrom::Current(size as i64))
    }
}