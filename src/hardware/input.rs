use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    EventPump, Sdl,
};

pub struct Input {
    event_pump: EventPump,
    focused: bool,
}

impl Input {
    pub fn new(sdl: &Sdl) -> Result<Self, String> {
        Ok(Self {
            event_pump: sdl.event_pump()?,
            focused: true,
        })
    }

    pub fn process_input(&mut self) -> Option<[bool; 16]> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    scancode: Some(Scancode::Escape),
                    ..
                } => return None,
                Event::Window {
                    win_event: WindowEvent::FocusLost,
                    ..
                } => self.focused = false,
                Event::Window {
                    win_event: WindowEvent::FocusGained,
                    ..
                } => self.focused = true,
                _ => {}
            }
        }
        if !self.focused {
            return Some([false; 16]);
        }
        let keyboard = self.event_pump.keyboard_state();
        let mapping = [
            Scancode::X,
            Scancode::Num1,
            Scancode::Num2,
            Scancode::Num3,
            Scancode::Q,
            Scancode::W,
            Scancode::E,
            Scancode::A,
            Scancode::S,
            Scancode::D,
            Scancode::Z,
            Scancode::C,
            Scancode::Num4,
            Scancode::R,
            Scancode::F,
            Scancode::V,
        ];
        let mut keys = mapping.map(|key| keyboard.is_scancode_pressed(key));
        keys[0xA] |= keyboard.is_scancode_pressed(Scancode::Y);
        Some(keys)
    }
}
