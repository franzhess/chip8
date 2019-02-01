use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::Sdl;
use sdl2::video::Window;

use crate::cpu::CHIP8_HEIGHT;
use crate::cpu::CHIP8_WIDTH;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 480;

pub struct Display {
  canvas: Canvas<Window>,
}

impl Display {
  pub fn new(sdl: &Sdl) -> Display {
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem.window("chip8-rust", WINDOW_WIDTH, WINDOW_HEIGHT)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(CHIP8_WIDTH as u32, CHIP8_HEIGHT as u32).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    Display {
      canvas,
    }
  }

  pub fn draw_screen(&mut self, screen_buffer: &[[bool; CHIP8_WIDTH]; CHIP8_HEIGHT]) {
    self.canvas.set_draw_color(Color::RGB(0, 0, 0));
    self.canvas.clear();

    self.canvas.set_draw_color(Color::RGB(80, 255, 80));
    for (y, line) in screen_buffer.iter().enumerate() {
      for (x, pixel) in line.iter().enumerate() {
        if *pixel {
          self.canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
        }
      }
    }

    self.canvas.present();
  }
}