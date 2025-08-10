use raylib::prelude::*;

pub struct Framebuffer {
    pub width:  u32,
    pub height: u32,
    buffer: Vec<Color>,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: vec![Color::BLACK; (width * height) as usize],
            current_color: Color::WHITE,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.buffer.resize((w * h) as usize, Color::BLACK);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx] = self.current_color;
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color);
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                d.draw_pixel(x as i32, y as i32, self.buffer[idx]);
            }
        }
    }
}
