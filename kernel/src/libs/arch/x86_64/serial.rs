#![allow(dead_code)]

use crate::libs::arch::x86_64::asm::*;

type Port = u16;
type Offset = u8;

pub const COM3: Port = 0x3E8;
pub const COM1: Port = 0x3F8;
pub const COM4: Port = 0x2E8;
pub const COM2: Port = 0x2F8;

const RX: Offset = 0;
const TX: Offset = 0;
const IER: Offset = 1; // Interrupt enable register, DLAB=0
const LB_BAUD: Offset = 0; // Divisor Latch Low Byte, DLAB=1
const HB_BAUD: Offset = 1; // Divisor Latch High Byte, DLAB=1
const INT_ID: Offset = 2;
const FIFO_CTRL: Offset = 2;
const LINE_CTRL: Offset = 3;
const MODEM_CTRL: Offset = 4;
const LINE_STATUS: Offset = 5;
const MODEM_STATUS: Offset = 6;
const SCRATCH: Offset = 7;

pub struct SerialSocket {
    pub port: Port,
}

impl SerialSocket {
    fn is_serial_transmit_empty(&self) -> bool {
        unsafe {
            (inb((self.port + LINE_STATUS as u16) as usize) & 0x20) != 0
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            while !self.is_serial_transmit_empty() {}
            outb((self.port + TX as u16) as usize, byte);
        }
    }

    pub fn new(port: Port) -> Option<Self> {
        unsafe {
            outb((port + IER as u16) as usize, 0x0); // Disable all interrupts
            outb((port + LINE_CTRL as u16) as usize, 0x80); // Enable DLAB
            outb((port + LB_BAUD as u16) as usize, 0x03); // Set baud rate to 38400 (low byte)
            outb((port + HB_BAUD as u16) as usize, 0x00);
            outb((port + LINE_CTRL as u16) as usize, 0x03); // 8 bits, no parity, one stop bit
            outb((port + FIFO_CTRL as u16) as usize, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
            outb((port + MODEM_CTRL as u16) as usize, 0x0B); // IRQs
            outb((port + MODEM_CTRL as u16) as usize, 0x1E); // Loopback mode

            // Test serial chip (send byte 0xAE and check if serial returns same byte)
            outb((port + TX as u16) as usize, 0xAE);
            let test_byte: u8 = inb((port + RX as u16) as usize);

            if test_byte != 0xAE {
                return None;
            }
            outb((port + MODEM_CTRL as u16) as usize, 0x0F); // Set normal operation mode
        }

        Some(SerialSocket { port })
    }
}
