use limine::framebuffer::Framebuffer;

use crate::libs::{drivers::logs::sinks::Sink, generic::parsers::psf::PsfFont};

static FONT_DATA: &[u8] = include_bytes!("../../../../../resources/fonts/zap-light16.psf");

// This is a naive, non-optimized VGA text mode driver.
// It is only intended to be used for displaying text when no memory management is available.
// It doesn't support scaling yet, since the goal is to just hand over control to a better log sink when possible
pub struct Vga<'a> {
    framebuffer: &'a Framebuffer<'a>,
    font: PsfFont,
    cursor_pos: (u32, u32),
}

impl<'a> Vga<'a> {
    pub fn clear(framebuffer: &'a Framebuffer<'a>, color: u32) {
        for i in (framebuffer.width() * 50)..framebuffer.width() * framebuffer.height() {
            unsafe {
                framebuffer
                    .addr()
                    .cast::<u32>()
                    .add(i as usize)
                    .write(color);
            };
        }
    }

    pub fn scroll(&mut self) {
        unsafe {
            let fb_raw: *mut u32 = self.framebuffer.addr().cast::<u32>();
            let fb_new_start: *mut u32 = self
                .framebuffer
                .addr()
                .cast::<u32>()
                .add(self.framebuffer.width() as usize * self.font.glyph_size.1 as usize);

            core::ptr::copy(
                fb_new_start,
                fb_raw,
                self.framebuffer.width() as usize
                    * (self.framebuffer.height() as usize - self.font.glyph_size.1 as usize),
            );
            self.clear_line(self.cursor_pos.1 as usize);
        };
    }

    pub fn clear_line(&mut self, line: usize) {
        unsafe {
            let fb_line: *mut u32 = self.framebuffer.addr().cast::<u32>().add(
                self.framebuffer.width() as usize
                    * (line as usize * self.font.glyph_size.1 as usize),
            );

            core::ptr::write_bytes(
                fb_line,
                0x0,
                self.framebuffer.width() as usize * self.font.glyph_size.1 as usize,
            );
        }
    }

    pub fn new(framebuffer: &'a Framebuffer<'a>) -> Self {
        let font = PsfFont::parse(FONT_DATA);

        if font.is_none() {
            Vga::clear(framebuffer, 0xFF0000FF);
            panic!("Failed to parse font data");
        }
        Self {
            framebuffer,
            font: font.unwrap(),
            cursor_pos: (0, 0),
        }
    }
}

impl<'a> Sink for Vga<'a> {
    fn putchar(&mut self, c: char) {
        if c == '\n' {
            if self.cursor_pos.1 as u64 * self.font.glyph_size.1 as u64
                >= (self.framebuffer.height() - (self.font.glyph_size.1 as u64))
            {
                self.scroll();
                self.cursor_pos.0 = 0;
            } else {
                self.cursor_pos.0 = 0;
                self.cursor_pos.1 += 1;
            }
            return;
        }

        let glyph_start: usize =
            (c as usize) * (self.font.glyph_stride as usize * self.font.glyph_size.1 as usize);
        let glyph_data: &[u8] = &self.font.glyphs[glyph_start..];

        for y in 0..self.font.glyph_size.1 {
            for x in 0..self.font.glyph_size.0 {
                let pixel_color: u32 = if glyph_data
                    [y as usize * self.font.glyph_stride as usize + (x as usize / 8)]
                    & (1 << 7 - (x % 8))
                    != 0
                {
                    0xFFFFFFFF
                } else {
                    0x00000000
                };

                let absolute_x: u64 =
                    (self.cursor_pos.0 as u64 * self.font.glyph_size.0 as u64) + x as u64;
                let absolute_y: u64 =
                    (self.cursor_pos.1 as u64 * self.font.glyph_size.1 as u64) + y as u64;

                unsafe {
                    self.framebuffer
                        .addr()
                        .cast::<u32>()
                        .add((absolute_y * self.framebuffer.width() + absolute_x) as usize)
                        .write(pixel_color);
                };
            }
        }
        self.cursor_pos.0 += 1;

        if self.cursor_pos.0 as u64 * self.font.glyph_size.0 as u64 >= self.framebuffer.width() {
            self.cursor_pos.0 = 0;
            self.cursor_pos.1 += 1;

            if self.cursor_pos.1 as u64 * self.font.glyph_size.1 as u64
                >= (self.framebuffer.height() - (self.font.glyph_size.1 as u64))
            {
                self.scroll();
                self.cursor_pos.0 = 0;
            }
        }
    }

    fn putstr(&mut self, s: &str) {
        for c in s.chars() {
            self.putchar(c);
        }
    }
}
