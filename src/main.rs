mod chip8;
mod rom;
mod display;
mod font;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;


fn main() {
  let sdl = sdl2::init().unwrap();

  let rom_loader = rom::RomLoader::load("roms/demos/Maze (alt) [David Winter, 199x].ch8");
  let mut display = display::Display::new(&sdl);

  let mut chip8 = chip8::Chip8::new();
  chip8.load(rom_loader.rom);

  let mut event_pump = sdl.event_pump().unwrap();

  while let Ok(input_state) = process_input(&mut event_pump) {
    let tick_result = chip8.tick(input_state);

    if tick_result.screen_changed {
      display.draw_screen(tick_result.screen_buffer);
    }
  }
}

fn process_input(event_pump: &mut EventPump) -> Result<[bool; 16], &str> {
  for event in event_pump.poll_iter() {
    match event {
      Event::Quit { .. } |
      Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        return Err("Esc")
      },
      _ => {}
    }
  }

  Ok([false; 16])
}