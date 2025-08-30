use crate::utils::byte_stream::ByteStream;

pub struct RwSectionHeader {
    pub section_type: u32,
    pub section_size: u32,
    pub version_number: u32,
}

pub struct RwFile<'a> {
    stream: ByteStream<'a>,
}

impl<'a> RwFile<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        RwFile {
            stream: ByteStream::new(buffer),
        }
    }

    pub fn read_section_header(&mut self) -> std::io::Result<RwSectionHeader> {
        let section_type = self.stream.read_u32()?;
        let section_size = self.stream.read_u32()?;
        let version_number = self.stream.read_u32()?;

        Ok(RwSectionHeader {
            section_type,
            section_size,
            version_number,
        })
    }

    pub fn get_stream(&mut self) -> &mut ByteStream<'a> {
        &mut self.stream
    }
}