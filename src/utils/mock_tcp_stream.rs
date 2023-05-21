use std::io::{self, Read, Write};

/// Has both read and write buffers to test if the messages are correctly sent
pub struct MockTcpStream {
    pub read_buffer: Vec<u8>,
    pub write_buffer: Vec<u8>,
}

impl MockTcpStream {
    /// Constructor for MockTcpStream
    pub fn new() -> MockTcpStream {
        MockTcpStream {
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
        }
    }
}

impl Read for MockTcpStream {
    /// Reads bytes from the stream until completing the buffer and returns how many bytes were read
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let quantity_read = self.read_buffer.as_slice().read(buf)?;

        self.read_buffer = self.read_buffer.split_off(quantity_read);
        Ok(quantity_read)
    }
}

impl Write for MockTcpStream {
    /// Writes the buffer value on the stream and returns how many bytes were written
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write_buffer.flush()
    }
}
