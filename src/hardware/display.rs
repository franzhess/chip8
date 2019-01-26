use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;

use chip8_rust::cpu::{CHIP8_HEIGHT, CHIP8_WIDTH};

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 480;

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl: &Sdl) -> Result<Display, String> {
        let video_subsystem = sdl.video()?;

        let window = video_subsystem
            .window("chip8-rust", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|error| error.to_string())?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|error| error.to_string())?;
        canvas
            .set_logical_size(CHIP8_WIDTH as u32, CHIP8_HEIGHT as u32)
            .map_err(|error| error.to_string())?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(Display { canvas })
    }

    pub fn draw_screen(
        &mut self,
        screen_buffer: &[[bool; CHIP8_WIDTH]; CHIP8_HEIGHT],
    ) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(80, 255, 80));
        for (y, line) in screen_buffer.iter().enumerate() {
            for (x, pixel) in line.iter().enumerate() {
                if *pixel {
                    self.canvas.draw_point(Point::new(x as i32, y as i32))?;
                }
            }
        }

        self.canvas.present();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::{input::Input, sound::Sound};
    use sdl2::{event::Event, pixels::PixelFormatEnum};

    #[test]
    fn desktop_renders_scaled_pixels_initializes_audio_and_handles_quit() {
        // This is the sole SDL test; dummy drivers avoid a display/audio device in CI.
        std::env::set_var("SDL_VIDEODRIVER", "dummy");
        std::env::set_var("SDL_AUDIODRIVER", "dummy");
        let sdl = sdl2::init().unwrap();
        let mut display = Display::new(&sdl).unwrap();
        let mut input = Input::new(&sdl).unwrap();
        let mut sound = Sound::new(&sdl).unwrap();
        let mut frame = [[false; CHIP8_WIDTH]; CHIP8_HEIGHT];
        frame[0][0] = true;
        display.draw_screen(&frame).unwrap();
        let pixels = display
            .canvas
            .read_pixels(None, PixelFormatEnum::RGB24)
            .unwrap();
        assert_eq!(
            pixels.len(),
            WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize * 3
        );
        let (rgb, remainder) = pixels.as_chunks::<3>();
        assert!(remainder.is_empty());
        assert_eq!(
            rgb.iter().filter(|pixel| **pixel == [80, 255, 80]).count(),
            225
        );
        assert_eq!(
            rgb.iter().filter(|pixel| **pixel == [0, 0, 0]).count(),
            960 * 480 - 225
        );
        sound.play();
        std::thread::sleep(std::time::Duration::from_millis(25));
        sound.stop();
        assert!(input.process_input().is_some());
        sdl.event()
            .unwrap()
            .push_event(Event::Quit { timestamp: 0 })
            .unwrap();
        assert!(input.process_input().is_none());
    }
}
