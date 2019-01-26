//! Render a ROM with the same CPU core used by the desktop application.
use chip8_rust::cpu::Chip8;
use std::{error::Error, fmt::Write};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let rom = args
        .next()
        .ok_or("usage: export_frame ROM OUTPUT.svg [STEPS]")?;
    let output = args.next().ok_or("missing output SVG path")?;
    let steps: usize = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(100);
    let mut cpu = Chip8::new();
    cpu.load(&std::fs::read(rom)?)?;
    for instruction in 0..steps {
        cpu.step([false; 16])?;
        if instruction % 8 == 7 {
            cpu.tick_timers();
        }
    }
    let mut svg = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"960\" height=\"480\" viewBox=\"0 0 64 32\" shape-rendering=\"crispEdges\" role=\"img\" aria-label=\"CHIP-8 emulator framebuffer\">\n<rect width=\"64\" height=\"32\" fill=\"black\"/>\n<g fill=\"#50ff50\">\n",
    );
    for (y, row) in cpu.screen().iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel {
                writeln!(svg, "<rect x=\"{x}\" y=\"{y}\" width=\"1\" height=\"1\"/>")?;
            }
        }
    }
    svg.push_str("</g>\n</svg>\n");
    std::fs::write(output, svg)?;
    Ok(())
}
