use crate::libs::{arch::x86_64::serial::{COM1, SerialSocket}, drivers::logs::sinks::Sink};

pub struct SerialSink {
    socket: SerialSocket,
}

impl SerialSink {
    pub fn new() -> Option<Self> {
        let socket = SerialSocket::new(COM1)?;

        Some(SerialSink {
            socket
        })
    }
}

impl Sink for SerialSink {
    fn putchar(&mut self, s: char) {
        self.socket.write_byte(s as u8);
    }

    fn putstr(&mut self, s: &str) {
        for c in s.chars() {
            self.putchar(c);
        }
    }
}
