mod chip8;
mod rom;
mod display;
mod font;
mod input;
mod sound;

fn main() {
  let sdl = sdl2::init().unwrap();

  let rom_loader = rom::RomLoader::load("roms/games/Soccer.ch8");
  let mut display = display::Display::new(&sdl);
  let mut input = input::Input::new(&sdl);
  let mut sound = sound::Sound::new(&sdl);

  let mut chip8 = chip8::Chip8::new();
  chip8.load(rom_loader.rom);

  while let Ok(input_state) = input.process_input() {
    let tick_result = chip8.tick(input_state);

    if tick_result.screen_changed {
      display.draw_screen(tick_result.screen_buffer);
    }

    if tick_result.play_sound {
      sound.play();
    } else {
      sound.stop();
    }
  }
}