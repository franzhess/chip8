use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;

pub struct Input {
  event_pump: EventPump,
  keys: [bool; 16],
}

impl Input {
  pub fn new(sdl: &Sdl) -> Input {
    Input {
      event_pump: sdl.event_pump().unwrap(),
      keys: [false; 16],
    }
  }

  // 1 2 3 C    1 2 3 4
  // 4 5 6 D    Q W E R
  // 7 8 9 E    A S D F
  // A 0 B F    Y X C V

  pub fn process_input(&mut self) -> Result<[bool; 16], &str> {
    for event in self.event_pump.poll_iter() {
      match event {
        Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return Err("Esc"),
        Event::KeyDown { keycode: Some(Keycode::X), .. } => self.keys[0] = true,
        Event::KeyUp { keycode: Some(Keycode::X), .. } => self.keys[0] = false,
        Event::KeyDown { keycode: Some(Keycode::Num1), .. } => self.keys[1] = true,
        Event::KeyUp { keycode: Some(Keycode::Num1), .. } => self.keys[1] = false,
        Event::KeyDown { keycode: Some(Keycode::Num2), .. } => self.keys[2] = true,
        Event::KeyUp { keycode: Some(Keycode::Num2), .. } => self.keys[2] = false,
        Event::KeyDown { keycode: Some(Keycode::Num3), .. } => self.keys[3] = true,
        Event::KeyUp { keycode: Some(Keycode::Num3), .. } => self.keys[3] = false,
        Event::KeyDown { keycode: Some(Keycode::Q), .. } => self.keys[4] = true,
        Event::KeyUp { keycode: Some(Keycode::Q), .. } => self.keys[4] = false,
        Event::KeyDown { keycode: Some(Keycode::W), .. } => self.keys[5] = true,
        Event::KeyUp { keycode: Some(Keycode::W), .. } => self.keys[5] = false,
        Event::KeyDown { keycode: Some(Keycode::E), .. } => self.keys[6] = true,
        Event::KeyUp { keycode: Some(Keycode::E), .. } => self.keys[6] = false,
        Event::KeyDown { keycode: Some(Keycode::A), .. } => self.keys[7] = true,
        Event::KeyUp { keycode: Some(Keycode::A), .. } => self.keys[7] = false,
        Event::KeyDown { keycode: Some(Keycode::S), .. } => self.keys[8] = true,
        Event::KeyUp { keycode: Some(Keycode::S), .. } => self.keys[8] = false,
        Event::KeyDown { keycode: Some(Keycode::D), .. } => self.keys[9] = true,
        Event::KeyUp { keycode: Some(Keycode::D), .. } => self.keys[9] = false,
        Event::KeyDown { keycode: Some(Keycode::Y), .. } => self.keys[10] = true,
        Event::KeyUp { keycode: Some(Keycode::Y), .. } => self.keys[10] = false,
        Event::KeyDown { keycode: Some(Keycode::C), .. } => self.keys[11] = true,
        Event::KeyUp { keycode: Some(Keycode::C), .. } => self.keys[11] = false,
        Event::KeyDown { keycode: Some(Keycode::Num4), .. } => self.keys[12] = true,
        Event::KeyUp { keycode: Some(Keycode::Num4), .. } => self.keys[12] = false,
        Event::KeyDown { keycode: Some(Keycode::R), .. } => self.keys[13] = true,
        Event::KeyUp { keycode: Some(Keycode::R), .. } => self.keys[13] = false,
        Event::KeyDown { keycode: Some(Keycode::F), .. } => self.keys[14] = true,
        Event::KeyUp { keycode: Some(Keycode::F), .. } => self.keys[14] = false,
        Event::KeyDown { keycode: Some(Keycode::V), .. } => self.keys[15] = true,
        Event::KeyUp { keycode: Some(Keycode::V), .. } => self.keys[15] = false,
        _ => {}
      }
    }

    Ok(self.keys.clone())
  }
}