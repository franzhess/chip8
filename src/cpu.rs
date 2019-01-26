use std::fmt;

#[cfg(test)]
mod tests;

const INSTRUCTION_SIZE: usize = 2;
const MEMORY_START: usize = 0x0200;
const MEMORY_SIZE: usize = 4096;
pub const MAX_ROM_SIZE: usize = MEMORY_SIZE - MEMORY_START;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmulatorError(pub String);

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EmulatorError {}

pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;

enum ProgramCounterAction {
    Increment,
    Skip,
    Jump(usize),
}

pub struct TickResult<'a> {
    pub screen_buffer: &'a [[bool; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub screen_changed: bool,
    pub play_sound: bool,
}

pub struct Chip8 {
    screen_buffer: [[bool; CHIP8_WIDTH]; CHIP8_HEIGHT],
    screen_changed: bool,

    input: [bool; 16],
    wait_for_input: bool,
    input_register: usize, // Destination register for a pending Fx0A instruction.

    memory: [u8; MEMORY_SIZE],
    v: [u8; 16],
    i: u16,
    delay_timer: u8,
    sound_timer: u8,
    program_counter: usize,
    stack: Vec<usize>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            screen_buffer: [[false; CHIP8_WIDTH]; CHIP8_HEIGHT],
            screen_changed: false,

            wait_for_input: false,
            input: [false; 16],
            input_register: 0,

            memory: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: MEMORY_START,
            stack: Vec::new(),
        }
    }

    /// Reset the machine and load a program at address 0x200.
    pub fn load(&mut self, rom: &[u8]) -> Result<(), EmulatorError> {
        if rom.is_empty() || rom.len() > MAX_ROM_SIZE {
            return Err(EmulatorError(format!(
                "ROM must contain 1..={MAX_ROM_SIZE} bytes (got {})",
                rom.len()
            )));
        }
        *self = Self::new();
        for (i, line) in FONT_SET.iter().enumerate() {
            for (j, byte) in line.iter().enumerate() {
                self.memory[i * 5 + j] = *byte;
            }
        }

        self.memory[MEMORY_START..MEMORY_START + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    /// Execute one instruction (or poll a pending key wait).
    pub fn step(&mut self, input: [bool; 16]) -> Result<TickResult<'_>, EmulatorError> {
        self.input = input;
        self.screen_changed = false;

        if !self.wait_for_input {
            self.validate_operation()?;
        }
        self.do_tick();

        Ok(TickResult {
            screen_buffer: &self.screen_buffer,
            screen_changed: self.screen_changed,
            play_sound: self.sound_timer > 0,
        })
    }

    /// Call once per 60 Hz timer tick, independently of instruction execution.
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn screen(&self) -> &[[bool; CHIP8_WIDTH]; CHIP8_HEIGHT] {
        &self.screen_buffer
    }

    pub fn sound_active(&self) -> bool {
        self.sound_timer > 0
    }

    fn validate_operation(&self) -> Result<(), EmulatorError> {
        let pc = self.program_counter;
        if pc >= MEMORY_SIZE - 1 {
            return Err(EmulatorError(format!(
                "Instruction fetch outside memory at 0x{pc:04X}"
            )));
        }
        let op = u16::from_be_bytes([self.memory[pc], self.memory[pc + 1]]);
        let x = ((op >> 8) & 15) as usize;
        let count = match op & 0xF000 {
            0xD000 => (op & 15) as usize,
            0xF000 => match op & 255 {
                0x33 => 3,
                0x55 | 0x65 => x + 1,
                _ => 0,
            },
            _ => 0,
        };
        if count > 0 && self.i as usize + count > MEMORY_SIZE {
            return Err(EmulatorError(format!(
                "Memory access outside RAM for opcode 0x{op:04X}"
            )));
        }
        if op == 0x00EE && self.stack.is_empty() {
            return Err(EmulatorError("Return with an empty stack".into()));
        }
        if op & 0xF000 == 0x2000 && self.stack.len() >= 16 {
            return Err(EmulatorError(
                "Subroutine stack overflow (16 entries)".into(),
            ));
        }
        let supported = match op & 0xF000 {
            0x0000 => true, // Legacy machine-code calls are ignored.
            0x5000 | 0x9000 => op & 15 == 0,
            0x8000 => matches!(op & 15, 0..=7 | 0xE),
            0xE000 => matches!(op & 255, 0x9E | 0xA1),
            0xF000 => matches!(
                op & 255,
                0x07 | 0x0A | 0x15 | 0x18 | 0x1E | 0x29 | 0x33 | 0x55 | 0x65
            ),
            _ => true,
        };
        if !supported {
            return Err(EmulatorError(format!(
                "Unsupported opcode 0x{op:04X} at 0x{pc:04X}"
            )));
        }
        Ok(())
    }

    fn do_tick(&mut self) {
        if self.wait_for_input {
            for (i, b) in self.input.iter().enumerate() {
                if *b {
                    self.v[self.input_register] = i as u8;

                    self.wait_for_input = false;
                    self.input_register = 0;
                    break;
                }
            }
        } else {
            match self.execute_operation() {
                ProgramCounterAction::Increment => self.program_counter += INSTRUCTION_SIZE,
                ProgramCounterAction::Skip => self.program_counter += INSTRUCTION_SIZE * 2,
                ProgramCounterAction::Jump(address) => self.program_counter = address,
            }
        }
    }

    fn execute_operation(&mut self) -> ProgramCounterAction {
        let op = (self.memory[self.program_counter] as u16) << 8
            | self.memory[self.program_counter + 1] as u16;

        let half_bytes = (
            ((op & 0xF000) >> 12) as u8,
            ((op & 0x0F00) >> 8) as u8,
            ((op & 0x00F0) >> 4) as u8,
            (op & 0x000F) as u8,
        );

        let addr = (op & 0x0FFF) as usize;
        let byte = (op & 0x00FF) as usize;

        let x = half_bytes.1 as usize;
        let y = half_bytes.2 as usize;
        let nibble = half_bytes.3 as usize;

        match half_bytes {
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),       //clear screen
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),       //return from subroutine
            (0x0, _, _, _) => self.op_0nnn(addr),         // Legacy machine-code call: ignored.
            (0x1, _, _, _) => self.op_1nnn(addr),         //jump to addr
            (0x2, _, _, _) => self.op_2nnn(addr),         // Call subroutine.
            (0x3, _, _, _) => self.op_3xkk(x, byte),      //skip if vx == kk
            (0x4, _, _, _) => self.op_4xkk(x, byte),      //skip if vx != kk
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),       //skip if vx == vy
            (0x6, _, _, _) => self.op_6xkk(x, byte),      //set vx = kk
            (0x7, _, _, _) => self.op_7xkk(x, byte),      //set vx = vx + kk
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),       //set vx = vy
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),       //set vx = vx | vy
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),       //set vx = vx & vy
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),       //set vx = vx ^ vy
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),       // Add with carry.
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),       // Subtract with no-borrow flag.
            (0x8, _, _, 0x6) => self.op_8xy6(x, y),       //set vx = vx / 2; if uneven vf = 1
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),       // Reverse subtract with no-borrow flag.
            (0x8, _, _, 0xE) => self.op_8xye(x, y), //set vx = vx * 2; if most significant bit = 1 then vf = 1
            (0x9, _, _, 0x0) => self.op_9xy0(x, y), //skip if vx != vy
            (0xA, _, _, _) => self.op_annn(addr),   //set i = nnn
            (0xB, _, _, _) => self.op_bnnn(addr),   //jump to nnn + v0
            (0xC, _, _, _) => self.op_cxkk(x, byte), // Random byte masked by kk.
            (0xD, _, _, _) => self.op_dxyn(x, y, nibble), // XOR sprite; VF records collision.
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),  //skip if key press == vx
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),  //skip if key not pressed == vx
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),  //vx = delay timer
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),  //wait for keypress and store in vx
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),  //set delay timer = vx
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),  //set sound timer = vx
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),  //set i = i + vx
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),  //set i = location of sprite for digit vx
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),  //set i bcd vx (i = 100, i+1 = 10, i+2 = 1)
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),  //write v0 to vx to memory starting at i
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),  //read v0 to vx from memory starting at i
            (_, _, _, _) => {
                unreachable!("opcode validated before execution")
            }
        }
    }

    fn op_0nnn(&mut self, _addr: usize) -> ProgramCounterAction {
        // Original interpreter machine-code routines are not emulated.
        ProgramCounterAction::Increment
    }

    fn op_00e0(&mut self) -> ProgramCounterAction {
        //clear screen
        self.screen_buffer = [[false; CHIP8_WIDTH]; CHIP8_HEIGHT];
        self.screen_changed = true;
        ProgramCounterAction::Increment
    }

    fn op_00ee(&mut self) -> ProgramCounterAction {
        //return from subroutine
        ProgramCounterAction::Jump(self.stack.pop().unwrap())
    }

    fn op_1nnn(&mut self, addr: usize) -> ProgramCounterAction {
        //jump to addr
        ProgramCounterAction::Jump(addr)
    }

    fn op_2nnn(&mut self, addr: usize) -> ProgramCounterAction {
        //call subroutine
        self.stack.push(self.program_counter + INSTRUCTION_SIZE);
        ProgramCounterAction::Jump(addr)
    }

    fn op_3xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction {
        //skip if vx == kk
        if self.v[x] == byte as u8 {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_4xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction {
        //skip if vx != kk
        if self.v[x] != byte as u8 {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //skip if vx == vy
        if self.v[x] == self.v[y] {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_6xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction {
        //set vx = kk
        self.v[x] = byte as u8;
        ProgramCounterAction::Increment
    }

    fn op_7xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction {
        //set vx = vx + kk
        self.v[x] = (self.v[x] as usize + byte) as u8;
        ProgramCounterAction::Increment
    }

    fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //set vx = vy
        self.v[x] = self.v[y];
        ProgramCounterAction::Increment
    }

    fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //set vx = vx | vy
        self.v[x] |= self.v[y];
        ProgramCounterAction::Increment
    }

    fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //set vx = vx & vy
        self.v[x] &= self.v[y];
        ProgramCounterAction::Increment
    }

    fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //set vx = vx ^ vy
        self.v[x] ^= self.v[y];
        ProgramCounterAction::Increment
    }

    fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        // Preserve operands before writing VF, which may alias Vx or Vy.
        let sum = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = (sum & 0x00FF) as u8;
        self.v[0xF] = u8::from(sum > 255);

        ProgramCounterAction::Increment
    }

    fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        // Equality does not borrow; underflow wraps to an 8-bit result.
        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[0xF] = u8::from(!borrow);

        ProgramCounterAction::Increment
    }

    fn op_8xy6(&mut self, x: usize, _y: usize) -> ProgramCounterAction {
        //set vx = vx / 2; if uneven vf = 1
        let value = self.v[x];
        self.v[x] = value >> 1;
        self.v[0xF] = value & 1;

        ProgramCounterAction::Increment
    }

    fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        // Reverse subtraction uses the same no-borrow convention.
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[0xF] = u8::from(!borrow);

        ProgramCounterAction::Increment
    }

    fn op_8xye(&mut self, x: usize, _y: usize) -> ProgramCounterAction {
        //set vx = vx * 2; if most significant bit = 1 then vf = 1
        let value = self.v[x];
        self.v[x] = value << 1;
        self.v[0xF] = value >> 7;

        ProgramCounterAction::Increment
    }

    fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction {
        //skip if vx != vy
        if self.v[x] != self.v[y] {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_annn(&mut self, addr: usize) -> ProgramCounterAction {
        //set i = nnn
        self.i = addr as u16;
        ProgramCounterAction::Increment
    }

    fn op_bnnn(&mut self, addr: usize) -> ProgramCounterAction {
        //jump to nnn + v0
        ProgramCounterAction::Jump(addr + self.v[0] as usize)
    }

    fn op_cxkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction {
        //set vx random byte & kkk
        self.v[x] = rand::random::<u8>() & byte as u8;

        ProgramCounterAction::Increment
    }

    fn op_dxyn(&mut self, x: usize, y: usize, nibble: usize) -> ProgramCounterAction {
        //display n-byte sprite starting at i at (vx,vy), vf = 1 if erased
        let mut deleted = false;

        for line in 0..nibble {
            let y = (self.v[y] as usize + line) % CHIP8_HEIGHT;
            let byte = self.memory[self.i as usize + line];

            for offset in 0..8 {
                let bit = ((byte >> (7 - offset)) & 1) != 0;
                let x = (self.v[x] as usize + offset) % CHIP8_WIDTH;

                let before = self.screen_buffer[y][x];
                self.screen_buffer[y][x] ^= bit;

                deleted = deleted || (before && !self.screen_buffer[y][x])
            }
        }

        self.v[0xF] = if deleted { 1 } else { 0 };
        self.screen_changed = true;

        ProgramCounterAction::Increment
    }

    fn op_ex9e(&mut self, x: usize) -> ProgramCounterAction {
        //skip if key press == vx
        if self.input[(self.v[x] & 0xF) as usize] {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_exa1(&mut self, x: usize) -> ProgramCounterAction {
        //skip if key not pressed == vx
        if !self.input[(self.v[x] & 0xF) as usize] {
            ProgramCounterAction::Skip
        } else {
            ProgramCounterAction::Increment
        }
    }

    fn op_fx07(&mut self, x: usize) -> ProgramCounterAction {
        //vx = delay timer
        self.v[x] = self.delay_timer;
        ProgramCounterAction::Increment
    }

    fn op_fx0a(&mut self, x: usize) -> ProgramCounterAction {
        //wait for keypress and store in vx
        self.wait_for_input = true;
        self.input_register = x;
        ProgramCounterAction::Increment
    }

    fn op_fx15(&mut self, x: usize) -> ProgramCounterAction {
        //set delay timer = vx
        self.delay_timer = self.v[x];
        ProgramCounterAction::Increment
    }

    fn op_fx18(&mut self, x: usize) -> ProgramCounterAction {
        //set sound timer = vx
        self.sound_timer = self.v[x];
        ProgramCounterAction::Increment
    }

    fn op_fx1e(&mut self, x: usize) -> ProgramCounterAction {
        //set i = i + vx
        self.i = self.i.wrapping_add(self.v[x] as u16);
        ProgramCounterAction::Increment
    }

    fn op_fx29(&mut self, x: usize) -> ProgramCounterAction {
        //set i = location of sprite for digit vx
        self.i = (self.v[x] & 0xF) as u16 * 5;
        ProgramCounterAction::Increment
    }

    fn op_fx33(&mut self, x: usize) -> ProgramCounterAction {
        //set i bcd vx, i = 100, i+1 = 10, i+2 = 1
        self.memory[self.i as usize] = self.v[x] / 100;
        self.memory[self.i as usize + 1] = (self.v[x] % 100) / 10;
        self.memory[self.i as usize + 2] = self.v[x] % 10;

        ProgramCounterAction::Increment
    }

    fn op_fx55(&mut self, x: usize) -> ProgramCounterAction {
        //write v0 to vx to memory starting at i
        for offset in 0..=x {
            self.memory[self.i as usize + offset] = self.v[offset];
        }

        ProgramCounterAction::Increment
    }

    fn op_fx65(&mut self, x: usize) -> ProgramCounterAction {
        //read v0 to vx from memory starting at i
        for offset in 0..=x {
            self.v[offset] = self.memory[self.i as usize + offset];
        }

        ProgramCounterAction::Increment
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

const DIGIT_0: [u8; 5] = [0xF0, 0x90, 0x90, 0x90, 0xF0];
const DIGIT_1: [u8; 5] = [0x20, 0x60, 0x20, 0x20, 0x70];
const DIGIT_2: [u8; 5] = [0xF0, 0x10, 0xF0, 0x80, 0xF0];
const DIGIT_3: [u8; 5] = [0xF0, 0x10, 0xF0, 0x10, 0xF0];
const DIGIT_4: [u8; 5] = [0x90, 0x90, 0xF0, 0x10, 0x10];
const DIGIT_5: [u8; 5] = [0xF0, 0x80, 0xF0, 0x10, 0xF0];
const DIGIT_6: [u8; 5] = [0xF0, 0x80, 0xF0, 0x90, 0xF0];
const DIGIT_7: [u8; 5] = [0xF0, 0x10, 0x20, 0x40, 0x40];
const DIGIT_8: [u8; 5] = [0xF0, 0x90, 0xF0, 0x90, 0xF0];
const DIGIT_9: [u8; 5] = [0xF0, 0x90, 0xF0, 0x10, 0xF0];
const DIGIT_A: [u8; 5] = [0xF0, 0x90, 0xF0, 0x90, 0x90];
const DIGIT_B: [u8; 5] = [0xE0, 0x90, 0xE0, 0x90, 0xE0];
const DIGIT_C: [u8; 5] = [0xF0, 0x80, 0x80, 0x80, 0xF0];
const DIGIT_D: [u8; 5] = [0xE0, 0x90, 0x90, 0x90, 0xE0];
const DIGIT_E: [u8; 5] = [0xF0, 0x80, 0xF0, 0x80, 0xF0];
const DIGIT_F: [u8; 5] = [0xF0, 0x80, 0xF0, 0x80, 0x80];

pub const FONT_SET: [[u8; 5]; 16] = [
    DIGIT_0, DIGIT_1, DIGIT_2, DIGIT_3, DIGIT_4, DIGIT_5, DIGIT_6, DIGIT_7, DIGIT_8, DIGIT_9,
    DIGIT_A, DIGIT_B, DIGIT_C, DIGIT_D, DIGIT_E, DIGIT_F,
];
