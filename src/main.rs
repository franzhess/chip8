mod hardware;

use chip8_rust::cpu::Chip8;
use std::error::Error;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const HELP: &str = "CHIP-8 — Rust / SDL2

Usage: chip8-rust [ROM]

With no ROM argument, runs the embedded CHIP-8 picture demo.
Keys: 1 2 3 4 / Q W E R / A S D F / Z X C V (Y also maps to Z)
Escape or closing the window exits.

Options:
  -h, --help    Show this help
";

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("chip8-rust: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let rom_path = args.next();
    if rom_path
        .as_deref()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        print!("{HELP}");
        return Ok(());
    }
    if args.next().is_some() {
        return Err("expected a single ROM path; use --help for usage".into());
    }
    let rom = match rom_path {
        Some(path) => hardware::rom::load(&PathBuf::from(path))?,
        None => include_bytes!("../roms/programs/Chip8 Picture.ch8").to_vec(),
    };
    let mut chip8 = Chip8::new();
    chip8.load(&rom)?;

    let sdl = sdl2::init().map_err(|error| format!("SDL initialization failed: {error}"))?;
    let mut display = hardware::display::Display::new(&sdl)?;
    let mut input = hardware::input::Input::new(&sdl)?;
    let mut sound = hardware::sound::Sound::new(&sdl)?;

    // CPU and timers have independent clocks; sleeping avoids a busy-spin loop.
    let instruction_period = Duration::from_secs_f64(1.0 / 500.0);
    let timer_period = Duration::from_secs_f64(1.0 / 60.0);
    let mut next_instruction = Instant::now();
    let mut next_timer = next_instruction + timer_period;
    while let Some(keys) = input.process_input() {
        let now = Instant::now();
        // Avoid a burst of instructions after the application has been suspended.
        if now.duration_since(next_instruction.min(now)) > Duration::from_millis(100) {
            next_instruction = now;
        }
        while next_timer <= now {
            chip8.tick_timers();
            next_timer += timer_period;
        }
        let mut redraw = false;
        while next_instruction <= now {
            redraw |= chip8.step(keys)?.screen_changed;
            next_instruction += instruction_period;
        }
        if redraw {
            display.draw_screen(chip8.screen())?;
        }
        if chip8.sound_active() {
            sound.play();
        } else {
            sound.stop();
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
