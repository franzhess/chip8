mod cpu;
mod hardware;

fn main() {
  let sdl = sdl2::init().unwrap();

  let rom_loader = hardware::rom::RomLoader::load("roms/programs/Chip8 Picture.ch8");
  let mut display = hardware::display::Display::new(&sdl);
  let mut input = hardware::input::Input::new(&sdl);
  let mut sound = hardware::sound::Sound::new(&sdl);

  let mut chip8 = cpu::Chip8::new();
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