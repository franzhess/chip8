use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::video::Window;

use crate::chip8::CHIP8_HEIGHT;
use crate::chip8::CHIP8_WIDTH;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 480;

pub struct Display {
  canvas: Canvas<Window>,
  scale_width: u32,
  scale_height: u32,
}

impl Display {
  pub fn new(sdl: &Sdl) -> Display {
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem.window("chip8-rust", WINDOW_WIDTH, WINDOW_HEIGHT)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    Display {
      canvas,
      scale_width: WINDOW_WIDTH / CHIP8_WIDTH as u32,
      scale_height: WINDOW_HEIGHT / CHIP8_HEIGHT as u32,
    }
  }

  pub fn draw_screen(&mut self, screen_buffer: &[[bool; CHIP8_WIDTH]; CHIP8_HEIGHT]) {
    self.canvas.set_draw_color(Color::RGB(0, 0, 0));
    self.canvas.clear();

    self.canvas.set_draw_color(Color::RGB(80, 255, 80));
    for (y, line) in screen_buffer.iter().enumerate() {
      for (x, pixel) in line.iter().enumerate() {
        if *pixel {
          self.canvas.fill_rect(sdl2::rect::Rect::new((self.scale_width * x as u32) as i32,
                                                      (self.scale_height * y as u32) as i32,
                                                      self.scale_width,
                                                      self.scale_height)).unwrap();
        }
      }
    }

    self.canvas.present();
  }
}