const MEMORY_ALIGNMENT: u16 = 2;
const MEMORY_START: usize = 0x0200;

enum ProgramCounterAction {
  Increment,
  Skip,
  Jump(u16),
}

pub struct Chip8 {
  pub screen_buffer: [u64; 32],  //screen is 64x32
  pub sound_interrupt: bool,
  pub wait_for_input: bool,
  pub input: [bool; 16],

  memory: [u8;4096],  //memory is 4k
  v: [u8; 16],  //16 8bit registers
  i: u16,  //one 16bit special register
  delay_timer: u8,  //delay counts down to zero
  sound_timer: u8,  //sound counts down to zero and plays sound
  program_counter: u16, //program counter
  stack: Vec<u16>,     //stack
}

impl Chip8 {
  pub fn new(rom: Vec<u8>) -> Chip8 {
    let mut memory = [0;4096];
    let mut start = MEMORY_START;

    for (i, byte) in rom.iter().enumerate() {
      memory[start + i] = *byte;
    }

    Chip8 {
      screen_buffer: [0; 32],
      sound_interrupt: false,
      wait_for_input: false,
      input: [false; 16],

      memory,
      v: [0; 16],
      i: 0,
      delay_timer: 0,
      sound_timer: 0,
      program_counter: MEMORY_START as u16,
      stack: Vec::new(),
    }
  }

  pub fn main_loop(&mut self) {
    self.process_timer();

    match self.execute_operation() {
      ProgramCounterAction::Increment => self.program_counter += MEMORY_ALIGNMENT,
      ProgramCounterAction::Skip => self.program_counter += MEMORY_ALIGNMENT * 2,
      ProgramCounterAction::Jump(adress) => self.program_counter = adress
    }
  }

  fn process_timer(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer = self.delay_timer - 1;
    }

    self.sound_interrupt = false;
    if self.sound_timer > 0 {
      self.sound_timer = self.sound_timer - 1;
      if self.sound_timer == 0 {
        self.sound_interrupt = true;
      }
    }
  }

  fn execute_operation(&mut self) -> ProgramCounterAction {
    let op = (self.memory[self.program_counter as usize] as u16) << 8 | self.memory[self.program_counter as usize + 1] as u16;
    println!("{:#06X} @ {:#06X}", op, &self.program_counter);

    let half_bytes = (
      ((op & 0xF000) >> 12) as u8,
      ((op & 0x0F00) >> 8) as u8,
      ((op & 0x00F0) >> 4) as u8,
      (op & 0x000F) as u8
    );

    let addr = (op & 0x0FFF) as usize;
    let x = ((op & 0x0F00) >> 8) as usize;
    let y = ((op & 0x00F0) >> 4) as usize;
    let nibble = (op & 0x000F) as usize;
    let byte = (op & 0x00FF) as usize;

    match half_bytes {
      (0x0, 0x0, 0xE, 0x0) => self.op_00e0(), //clear screen
      (0x0, 0x0, 0xE, 0xE) => self.op_00ee(), //return from subroutine
      (0x0,   _,   _,   _) => self.op_0nnn(addr), //system routing - NOself.op
      (0x1,   _,   _,   _) => self.op_1nnn(addr), //jump to addr
      (0x2,   _,   _,   _) => self.op_2nnn(addr), //call add (subroutine)
      (0x3,   _,   _,   _) => self.op_3xkk(x, byte), //skip if vx == kk
      (0x4,   _,   _,   _) => self.op_4xkk(x, byte), //skip if vx != kk
      (0x5,   _,   _, 0x0) => self.op_5xy0(x, y), //skip if vx == vy
      (0x6,   _,   _,   _) => self.op_6xkk(x, byte), //set vx = kk
      (0x7,   _,   _,   _) => self.op_7xkk(x, byte), //set vx = vx + kk
      (0x8,   _,   _, 0x0) => self.op_8xy0(x, y), //set vx = vy
      (0x8,   _,   _, 0x1) => self.op_8xy1(x, y), //set vx = vx | vy
      (0x8,   _,   _, 0x2) => self.op_8xy2(x, y), //set vx = vx & vy
      (0x8,   _,   _, 0x3) => self.op_8xy3(x, y), //set vx = vx ^ vy
      (0x8,   _,   _, 0x4) => self.op_8xy4(x, y), //set vx = vx + vy, only 8 bits are kept, vf = 1 if > 256 else 0
      (0x8,   _,   _, 0x5) => self.op_8xy5(x, y), //set vx = vx - vy, if vx > vy vf = 1
      (0x8,   _,   _, 0x6) => self.op_8xy6(x, y), //set vx = vx / 2; if uneven vf = 1
      (0x8,   _,   _, 0x7) => self.op_8xy7(x, y), //set vx = vy - vx; if vy > vx vf = 1
      (0x8,   _,   _, 0xE) => self.op_8xye(x, y), //set vx = vx * 2; if most significant bit = 1 then vf = 1
      (0x9,   _,   _, 0x0) => self.op_9xy0(x, y), //skip if vx != vy
      (0xA,   _,   _,   _) => self.op_annn(addr), //set i = nnn
      (0xB,   _,   _,   _) => self.op_bnnn(addr), //jump to nnn + v0
      (0xC,   _,   _,   _) => self.op_cxkk(x, byte), //set vx random byte + kkk
      (0xD,   _,   _,   _) => self.op_dxyn(x, y, nibble), //display n-byte spring starting at i at (vx,vy), vf = 1 if erased
      (0xE,   _, 0x9, 0xE) => self.op_ex9e(x), //skip if key press == vx
      (0xE,   _, 0xA, 0x1) => self.op_exa1(x), //skip if key not pressed == vx
      (0xF,   _, 0x0, 0x7) => self.op_fx07(x), //vx = delay timer
      (0xF,   _, 0x0, 0xA) => self.op_fx0a(x), //wait for keypress and store in vx
      (0xF,   _, 0x1, 0x5) => self.op_fx15(x), //set delay timer = vx
      (0xF,   _, 0x1, 0x8) => self.op_fx18(x), //set sound timer = vx
      (0xF,   _, 0x1, 0xE) => self.op_fx1e(x), //set i = i + vx
      (0xF,   _, 0x2, 0x9) => self.op_fx29(x), //set i = location of sprite for digit vx
      (0xF,   _, 0x3, 0x3) => self.op_fx33(x), //set i bcd vx (i = 100, i+1 = 10, i+2 = 1)
      (0xF,   _, 0x5, 0x5) => self.op_fx55(x), //write v0 to vx to memory starting at i
      (0xF,   _, 0x6, 0x5) => self.op_fx65(x), //read v0 to vx from memory starting at i
      (  _,   _,   _,   _) => {
        println!("Unrecognized command {:X} @ {:X}", op, self.program_counter);
        ProgramCounterAction::Increment
      }
    }
  }

  fn op_0nnn(&mut self, addr: usize)-> ProgramCounterAction { //system routing - NOOP
    ProgramCounterAction::Increment
  }

  fn op_00e0(&mut self, ) -> ProgramCounterAction { //clear screen
    self.screen_buffer = [0;32];
    ProgramCounterAction::Increment
  }

  fn op_00ee(&mut self, ) -> ProgramCounterAction { //return from subroutine
    ProgramCounterAction::Jump(self.stack.pop().unwrap())
  }

  fn op_1nnn(&mut self, addr: usize) -> ProgramCounterAction { //jump to addr
    ProgramCounterAction::Jump(addr as u16)
  }

  fn op_2nnn(&mut self, addr: usize) -> ProgramCounterAction { //call subroutine
    self.stack.push(self.program_counter + 1);
    ProgramCounterAction::Jump(addr as u16)
  }

  fn op_3xkk(&mut self, x: usize, byte: usize)-> ProgramCounterAction { //skip if vx == kk
    if self.v[x] == byte as u8 {
      ProgramCounterAction::Skip
    } else {
      ProgramCounterAction::Increment
    }
  }

  fn op_4xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction { //skip if vx != kk
    if self.v[x] != byte as u8 {
      ProgramCounterAction::Skip
    } else {
      ProgramCounterAction::Increment
    }
  }

  fn op_5xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction { //skip if vx == vy
    if self.v[x] == self.v[y] {
      ProgramCounterAction::Skip
    } else {
      ProgramCounterAction::Increment
    }
  }

  fn op_6xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction { //set vx = kk
    self.v[x] = byte as u8;
    ProgramCounterAction::Increment
  }

  fn op_7xkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction { //set vx = vx + kk
    self.v[x] = self.v[x] + byte as u8;
    ProgramCounterAction::Increment

  }

  fn op_8xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vy
    self.v[x] = self.v[y];
    ProgramCounterAction::Increment

  }

  fn op_8xy1(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx | vy
    self.v[x] = self.v[x] | self.v[y];
    ProgramCounterAction::Increment
  }

  fn op_8xy2(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx & vy
    self.v[x] = self.v[x] & self.v[y];
    ProgramCounterAction::Increment
  }

  fn op_8xy3(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx ^ vy
    self.v[x] = self.v[x] ^ self.v[y];
    ProgramCounterAction::Increment
  }

  fn op_8xy4(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx + vy, only 8 bits are kept-> ProgramCounterAction { vf = 1 if > 256 else 0
    self.v[x] = self.v[x] + self.v[y];
    if self.v[x] > 255 {
      self.v[0xF] = 1;
    } else {
      self.v[0xF] = 0;
    }
    self.v[x] = self.v[x] & 0x00FF;

    ProgramCounterAction::Increment
  }

  fn op_8xy5(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx - vy, if vx > vy vf = 1
    ProgramCounterAction::Increment

  }

  fn op_8xy6(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx / 2; if uneven vf = 1
    ProgramCounterAction::Increment

  }

  fn op_8xy7(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vy - vx; if vy > vx vf = 1
    ProgramCounterAction::Increment

  }

  fn op_8xye(&mut self, x: usize, y: usize) -> ProgramCounterAction { //set vx = vx * 2; if most significant bit = 1 then vf = 1
    ProgramCounterAction::Increment

  }

  fn op_9xy0(&mut self, x: usize, y: usize) -> ProgramCounterAction { //skip if vx != vy
    if self.v[x] != self.v[y] {
      ProgramCounterAction::Skip
    } else {
      ProgramCounterAction::Increment
    }
  }

  fn op_annn(&mut self, addr: usize) -> ProgramCounterAction { //set i = nnn
    self.i = addr as u16;
    ProgramCounterAction::Increment
  }

  fn op_bnnn(&mut self, addr: usize) -> ProgramCounterAction { //jump to nnn + v0
    ProgramCounterAction::Jump(addr as u16 + self.v[0] as u16)

  }

  fn op_cxkk(&mut self, x: usize, byte: usize) -> ProgramCounterAction { //set vx random byte + kkk
    ProgramCounterAction::Increment

  }

  fn op_dxyn(&mut self, x: usize, y: usize, nibble: usize) -> ProgramCounterAction { //display n-byte spring starting at i at (vx,vy), vf = 1 if erased
    ProgramCounterAction::Increment

  }

  fn op_ex9e(&mut self, x: usize) -> ProgramCounterAction { //skip if key press == vx
    ProgramCounterAction::Increment

  }

  fn op_exa1(&mut self, x: usize) -> ProgramCounterAction { //skip if key not pressed == vx
    ProgramCounterAction::Increment

  }

  fn op_fx07(&mut self, x: usize) -> ProgramCounterAction { //vx = delay timer
    ProgramCounterAction::Increment

  }

  fn op_fx0a(&mut self, x: usize) -> ProgramCounterAction { //wait for keypress and store in vx
    ProgramCounterAction::Increment

  }

  fn op_fx15(&mut self, x: usize) -> ProgramCounterAction { //set delay timer = vx
    ProgramCounterAction::Increment

  }

  fn op_fx18(&mut self, x: usize) -> ProgramCounterAction { //set sound timer = vx
    ProgramCounterAction::Increment

  }

  fn op_fx1e(&mut self, x: usize) -> ProgramCounterAction { //set i = i + vx
    ProgramCounterAction::Increment

  }

  fn op_fx29(&mut self, x: usize) -> ProgramCounterAction { //set i = location of sprite for digit vx
    ProgramCounterAction::Increment

  }

  fn op_fx33(&mut self, x: usize) -> ProgramCounterAction { //set i bcd vx, i = 100, i+1 = 10, i+2 = 1
    ProgramCounterAction::Increment

  }

  fn op_fx55(&mut self, x: usize) -> ProgramCounterAction { //write v0 to vx to memory starting at i
    ProgramCounterAction::Increment

  }

  fn op_fx65(&mut self, x: usize) -> ProgramCounterAction { //read v0 to vx from memory starting at i
    ProgramCounterAction::Increment
  }
}
