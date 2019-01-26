mod chip8;

use chip8::Chip8;
use std::{thread, time};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
  let mut rom = fs::read("roms/demos/Trip8 Demo (2008) [Revival Studios].ch8").expect("Failed to load file!");
  let mut chip8 = Chip8::new(rom);

  let running = Arc::new(AtomicBool::new(true));
  let running_clone = running.clone();

  ctrlc::set_handler(move || {
    running_clone.store(false, Ordering::SeqCst)
  }).expect("Error setting Ctrl-C handler");


  let duration = time::Duration::from_millis(16);
  while running.load(Ordering::SeqCst) {
    let now = time::Instant::now();

    chip8.input = process_input(&chip8.wait_for_input);

    chip8.main_loop();

    draw_screen(&chip8.screen_buffer);
    if chip8.sound_interrupt {
      play_sound();
    }

    let elapsed = now.elapsed();
    if elapsed < duration { //clamp to 60Hz
      thread::sleep(duration - elapsed);
    }
  }
}

fn draw_screen(screen_buffer: &[u64]) {

}

fn play_sound() {

}

fn process_input(wait: &bool) -> [bool;16] {
  [false;16]
}