# CHIP-8 in Rust

[![CI](https://github.com/franzhess/chip8/actions/workflows/ci.yml/badge.svg)](https://github.com/franzhess/chip8/actions/workflows/ci.yml)
![Rust](https://img.shields.io/badge/Rust-2021_edition-orange?logo=rust)
![Display](https://img.shields.io/badge/display-64%C3%9732-50ff50)

A CHIP-8 emulator written in Rust, with an SDL2 desktop frontend and an independently testable interpreter core. Load a ROM, play with the hexadecimal keypad, and hear the sound timer through a 440 Hz square wave.

![CHIP-8 picture demo rendered by the emulator](docs/chip8-preview.svg)

*The bundled picture demo, exported directly from the emulator's 64×32 framebuffer using the same green-on-black palette as the desktop window.*

## About the project

CHIP-8 is a small interpreted language originally used on 1970s microcomputers. Its compact instruction set makes it a useful introduction to emulation: instruction decoding, registers, memory, subroutines, sprite drawing, input, and timers all fit into a manageable system.

This repository began as an early Rust project. The refresh retains its opcode-oriented implementation while adding a reusable core, regression tests, checked execution, a command-line launcher, and automated checks.

### Features

- Standard CHIP-8 instruction set with 4 KiB of RAM, 16 registers, and a 16-entry call stack.
- 64×32 XOR sprite rendering, collision detection, and a 960×480 SDL2 window.
- Hexadecimal keypad mapped to a familiar four-row keyboard layout.
- 500 instructions per second and independent 60 Hz delay/sound timers.
- Audio driven by the sound timer, including while waiting for input.
- ROM selection with readable errors for invalid files and execution faults.
- Headless CPU tests and a reproducible framebuffer export example.

## Quick start

Install [Rust](https://rustup.rs/) (current stable) and SDL2 development libraries with `pkg-config`:

**macOS (Homebrew)**

```sh
brew install sdl2 pkg-config
```

**Ubuntu / Debian**

```sh
sudo apt-get update
sudo apt-get install libsdl2-dev pkg-config
```

Then clone and run:

```sh
git clone https://github.com/franzhess/chip8.git
cd chip8
cargo run --locked --release
```

With no argument, the application runs the embedded picture demo, so the built executable also works outside the repository directory.

To run another ROM, pass its path after Cargo's `--` separator:

```sh
cargo run --locked --release -- "roms/programs/IBM Logo.ch8"
cargo run --locked --release -- "roms/games/Pong [Paul Vervalin, 1990].ch8"
cargo run --locked -- --help
```

The desktop build uses `pkg-config` to locate SDL2. If SDL2 is installed in a custom location, add its `lib/pkgconfig` directory to `PKG_CONFIG_PATH`. The historical `SDL2.dll` in this repository is not used by the current build; the documented and CI-tested desktop platforms are macOS and Linux.

## Controls

```text
CHIP-8 keypad         Keyboard
┌─────────┐          ┌─────────┐
│ 1 2 3 C │          │ 1 2 3 4 │
│ 4 5 6 D │          │ Q W E R │
│ 7 8 9 E │          │ A S D F │
│ A 0 B F │          │ Z X C V │
└─────────┘          └─────────┘
```

**Escape** or closing the window exits. **Y** also maps to CHIP-8 key **A**, preserving the original mapping for QWERTZ keyboards. Physical key positions are used. Game-specific controls are described in the companion `.txt` files where available.

## Architecture

| Location | Responsibility |
| --- | --- |
| [`src/cpu.rs`](src/cpu.rs) | Machine state, opcode decoding, checked instruction execution, framebuffer, timers |
| [`src/cpu/tests.rs`](src/cpu/tests.rs) | CPU regression tests and a bundled ROM smoke test |
| [`src/lib.rs`](src/lib.rs) | Headless library entry point |
| [`src/main.rs`](src/main.rs) | ROM selection and independently scheduled CPU/timer loop |
| [`src/hardware/`](src/hardware/) | SDL2 display, input, audio, and file loading |
| [`examples/export_frame.rs`](examples/export_frame.rs) | Headless SVG framebuffer export |

The CPU has no SDL or wall-clock dependency. `Chip8::step(keys)` executes one instruction and returns display/sound state; `Chip8::tick_timers()` advances timers once. The frontend supplies the clocks and sleeps between iterations rather than busy-spinning.

## Compatibility

The emulator targets **standard 64×32 CHIP-8**, with a single explicit set of quirks:

| Behavior | Choice |
| --- | --- |
| `8xy6` / `8xyE` shifts | Shift `Vx`; ignore `Vy` |
| `Fx55` / `Fx65` register transfers | Include `Vx`; leave `I` unchanged |
| `Bnnn` jump offset | Add `V0` |
| Sprite edges | Wrap horizontally and vertically |
| Logic instructions | Leave `VF` unchanged unless it is the destination |
| `Fx1E` index addition | Leave `VF` unchanged |
| `Fx0A` key wait | Accept a held key; lowest pressed key wins |
| Key/font lookup | Use the low nibble of `Vx` |
| `0nnn` machine-code calls | Ignore |

CHIP-8 dialects differ, so ROMs expecting other quirks may behave differently. Super-CHIP, XO-CHIP, and the original VIP 64×64 mode are not implemented. In particular, the historical collection in `roms/hires/` is outside this emulator's supported display mode.

## Development

Run the core tests without installing SDL2:

```sh
cargo test --locked --no-default-features
```

With SDL2 installed, run the full checks:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

Tests cover wrapping arithmetic and `VF` aliasing, inclusive register transfers at the top of RAM, sprite wrapping/collisions, calls and skips, keypad waits, independent timers, BCD/fonts, ROM loading, and invalid execution. The IBM-logo smoke test checks the rendered pixel count and frame stability. A desktop smoke test uses SDL's dummy drivers to verify scaled green/black pixels, initialize audio playback, and process a quit event. GitHub Actions checks the headless core and desktop builds on Linux and macOS.

Regenerate the README preview without SDL2:

```sh
cargo run --locked --no-default-features --example export_frame -- \
  "roms/programs/Chip8 Picture.ch8" docs/chip8-preview.svg 100
```

The exporter takes `ROM OUTPUT.svg [STEPS]`. It is useful for static demos; it supplies no keypad input and advances timers every eight instructions.

## ROMs and references

The historical ROM collection includes games, demos, and diagnostic programs. Author names and dates are retained in filenames, with original companion notes where available; see [`roms/README.md`](roms/README.md). These programs are separate works from the emulator.

Useful references for the instruction set and its variants:

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [A guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite)
