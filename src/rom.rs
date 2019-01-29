use std::fs;

pub struct RomLoader {
  pub rom: Vec<u8>,
}

impl RomLoader {
  pub fn load(file_name: &str) -> RomLoader {
    let rom = fs::read(file_name).expect("Failed to load file!");

    RomLoader {
      rom
    }
  }
}