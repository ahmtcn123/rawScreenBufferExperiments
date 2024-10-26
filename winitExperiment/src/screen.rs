use fontdue::layout::GlyphRasterConfig;
use rand::Rng;
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
};

#[derive(Default, Clone, Copy, Debug)]
pub struct CharBoundary {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
}

use crate::cursor::{Cursor, Font};

#[derive(Clone, Debug)]
pub struct ScreenBuffer {
    pub height: usize,
    pub width: usize,
    pub buffer: Vec<u32>,
}

impl Read for ScreenBuffer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut i = 0;
        for b in buf.iter_mut() {
            *b = self.buffer[i] as u8;
            i += 1;
        }
        Ok(buf.len())
    }
}

impl Write for ScreenBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut i = 0;
        for b in buf.iter() {
            self.buffer[i] = *b as u32;
            i += 1;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

impl Color {
    pub fn from_rgb(r: u32, g: u32, b: u32) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: 255,
        }
    }

    pub fn from_rgba(r: u32, g: u32, b: u32, a: u32) -> Color {
        Color {
            red: r,
            green: g,
            blue: b,
            alpha: 255,
        }
    }

    pub fn from_hex(hex: u32) -> Color {
        Color {
            red: ((hex >> 16) & 0xFF) as u32,
            green: ((hex >> 8) & 0xFF) as u32,
            blue: (hex & 0xFF) as u32,
            alpha: 255,
        }
    }

    pub fn rand() -> Color {
        Color {
            red: rand::thread_rng().gen_range(0..255),
            green: rand::thread_rng().gen_range(0..255),
            blue: rand::thread_rng().gen_range(0..255),
            alpha: 255,
        }
    }

    pub fn red() -> Color {
        Color::from_rgb(255, 0, 0)
    }

    pub fn green() -> Color {
        Color::from_rgb(0, 255, 0)
    }

    pub fn blue() -> Color {
        Color::from_rgb(0, 0, 255)
    }

    pub fn to_hex_rgb(&self) -> u32 {
        (self.red << 16) | (self.green << 8) | self.blue
    }

    pub fn to_hex_rgba(&self) -> u32 {
        (self.red << 16) | (self.green << 8) | self.blue | self.alpha
    }
}
pub struct Boundaries {
    pub start_x: usize,
    pub start_y: usize,
    pub width: usize,
    pub height: usize,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> ScreenBuffer {
        ScreenBuffer {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    pub fn calc_buf_pos(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn draw_image(&mut self, image: Vec<u8>, w: usize, h: usize, color_type: u8) {
        let mut px = 0;
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w) as usize;
                let idy = (idx + x as usize) + px;
                let r = idy;
                let g = idy + 1;
                let b = idy + 2;
                let a = if color_type == 0 { 255 } else { idy + 3 };
                px += if color_type == 0 { 2 } else { 3 };
                let pixel = if color_type == 0 {
                    vec![image[r], image[g], image[b]]
                } else {
                    vec![image[r], image[g], image[b], image[a]]
                };

                ////let pixel = img_data[c];
                ////let pixel = rgbchunks.next().unwrap();

                let color = if color_type == 0 {
                    Color::from_rgb(pixel[0] as u32, pixel[1] as u32, pixel[2] as u32)
                } else {
                    Color::from_rgba(
                        pixel[0] as u32,
                        pixel[1] as u32,
                        pixel[2] as u32,
                        pixel[3] as u32,
                    )
                };

                if color_type == 0 {
                    self.put_pixel(x as usize, y as usize, color);
                } else {
                    self.put_pixel_a(x + 1 as usize, y + 1 as usize, color);
                }
            }
        }
    }

    pub fn draw_line(
        &mut self,
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize,
        color: Color,
    ) {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        for x in start_x..end_x {
            let y = start_y + dy * (x - start_x) / dx;
            self.put_pixel(x, y, Color::from_rgb(color.red, color.green, color.blue));
        }
        for y in start_y..end_y {
            let x = start_y + dx * (y - start_x) / dy;
            self.put_pixel(x, y, Color::from_rgb(color.red, color.green, color.blue));
        }
    }

    pub fn draw_rect(
        &mut self,
        start_x: usize,
        start_y: usize,
        width: usize,
        height: usize,
        color: Color,
    ) {
        //self.draw_line(start_x, start_y, start_x + width, start_y, color.clone()); //TOP
        //self.draw_line(start_x, start_y, start_x, start_y + height, color); //LEFT

        for y in start_y..(start_y + height) {
            for x in start_x..(start_x + width) {
                self.put_pixel(x, y, color);
            }
        }
    }

    pub fn draw_char(
        &mut self,
        chr: char,
        x: usize,
        y: usize,
        color: Color,
        background_color: Color,
        font: &Font,
    ) {
        let (metrics, bitmap) = font.builded.rasterize(chr, font.font_size);
        let mut current_x = x;
        let mut current_y = y;

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let char_s = bitmap[x + y * metrics.width];

                let mut char_color = Color::from_rgb(char_s as u32, char_s as u32, char_s as u32);

                if char_color.red != 0 && char_color.green != 0 && char_color.blue != 0 {
                    char_color = color
                } else if char_color.red == 0 && char_color.green == 0 && char_color.blue == 0 {
                    char_color = background_color;
                }

                self.put_pixel(
                    current_x,
                    current_y
                        + ((metrics.ymin * -1) as usize)
                        + (if (font.font_size as usize) < metrics.height {
                            0
                        } else {
                            (font.font_size as usize) - metrics.height
                        }),
                    char_color,
                );
                current_x += 1;
            }
            current_y += 1;
            current_x = x;
        }
    }

    pub fn draw_bitmap(&mut self, bitmap: Vec<Vec<u32>>) {
        for y in 0..bitmap.len() {
            for x in 0..bitmap[y].len() {
                let buf_pos = self.calc_buf_pos(x, y);
                self.buffer[buf_pos] = bitmap[y][x]
            }
        }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        let buf_pos = self.calc_buf_pos(x, y);
        if self.buffer.len() > buf_pos {
            self.buffer[buf_pos] = color.to_hex_rgb();
        }
    }

    pub fn put_pixel_a(&mut self, x: usize, y: usize, color: Color) {
        let buf_pos = self.calc_buf_pos(x, y);
        if self.buffer.len() > buf_pos {
            self.buffer[buf_pos] = color.to_hex_rgba();
        }
    }

    pub fn clear(&mut self, color: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer = vec![0; width * height];
    }

    pub fn render(&mut self, target_buffer: &[u32]) {
        /* self. */
    }
}
