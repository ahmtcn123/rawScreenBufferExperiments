use crate::{
    cursor::{Boundaries, Cursor, Font},
    screen::ScreenBuffer,
};

pub struct Window {
    pub cursor: Cursor,
    pub draw_buffer: ScreenBuffer,
    pub position: (usize, usize),
    pub height: usize,
    pub width: usize,
}

impl Window {
    pub fn new(font: Font, position: (usize, usize), height: usize, width: usize) -> Window {
        Window {
            cursor: Cursor::new(
                font,
                Boundaries {
                    start_x: position.0,
                    start_y: position.1,
                    width,
                    height,
                },
            ),
            position,
            draw_buffer: ScreenBuffer::new(width, height),
            height,
            width,
        }
    }

    pub fn resize(&mut self, height: usize, width: usize) {
        self.height = height;
        self.width = width;
        self.cursor.boundaries = Boundaries {
            start_x: self.position.0,
            start_y: self.position.1,
            width,
            height,
        };
        self.draw_buffer.resize(width, height);
    }

    pub fn render_on_screen(&self, screen_buffer: &mut ScreenBuffer) {
        self.cursor.render(screen_buffer);
        let start_pos = screen_buffer.calc_buf_pos(self.position.0, self.position.1);
        screen_buffer.buffer.splice(
            start_pos..(start_pos + self.draw_buffer.buffer.len()).min(screen_buffer.buffer.len()),
            self.draw_buffer.buffer.iter().cloned(),
        );
    }
}
