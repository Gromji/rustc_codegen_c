struct Buffer {
    buf: [u8; 1024],
}

impl Buffer {
    fn new() -> Self {
        Self { buf: [0; 1024] }
    }

    fn write(&mut self, data: &[u8]) {
        let index = 0;
        while index < data.len() {
            self.buf[index] = data[index];
        }
    }

    fn read(&self) -> &[u8] {
        &self.buf
    }
}

fn main() {
    let mut buf = Buffer::new();
    buf.write(b"hello");
}
