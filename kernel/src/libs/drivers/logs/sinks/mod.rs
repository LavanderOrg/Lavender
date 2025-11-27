pub mod vga;
pub mod serial;

pub trait Sink {
    fn putchar(&mut self, s: char);
    fn putstr(&mut self, s: &str);
}
