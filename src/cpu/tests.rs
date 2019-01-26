use super::*;

fn machine(ops: &[u16]) -> Chip8 {
    let rom: Vec<u8> = ops.iter().flat_map(|op| op.to_be_bytes()).collect();
    let mut cpu = Chip8::new();
    cpu.load(&rom).unwrap();
    cpu
}

fn step(cpu: &mut Chip8, count: usize) {
    for _ in 0..count {
        cpu.step([false; 16]).unwrap();
    }
}

#[test]
fn subtraction_wraps_and_equality_does_not_borrow() {
    for (opcode, a, b, expected, flag) in [
        (0x8125, 3, 5, 254, 0),
        (0x8125, 5, 3, 2, 1),
        (0x8125, 5, 5, 0, 1),
        (0x8127, 5, 3, 254, 0),
        (0x8127, 3, 5, 2, 1),
        (0x8127, 5, 5, 0, 1),
    ] {
        let mut cpu = machine(&[0x6100 | a, 0x6200 | b, opcode]);
        step(&mut cpu, 3);
        assert_eq!((cpu.v[1], cpu.v[15]), (expected, flag));
    }
}

#[test]
fn arithmetic_uses_original_operands_and_writes_flag_last() {
    for (opcode, a, b, expected, flag) in [
        (0x8124, 255, 2, 1, 1),
        (0x81F4, 255, 1, 0, 1),
        (0x8F24, 255, 2, 1, 1),
        (0x8F25, 1, 2, 0, 0),
        (0x8F27, 2, 1, 0, 0),
        (0x8F26, 3, 0, 1, 1),
        (0x8F2E, 128, 0, 1, 1),
    ] {
        let mut cpu = machine(&[opcode]);
        let x = ((opcode >> 8) & 15) as usize;
        let y = ((opcode >> 4) & 15) as usize;
        cpu.v[x] = a;
        cpu.v[y] = b;
        step(&mut cpu, 1);
        assert_eq!((cpu.v[x], cpu.v[15]), (expected, flag), "{opcode:04X}");
    }
}

#[test]
fn immediate_add_wraps_without_changing_flag() {
    let mut cpu = machine(&[0x61FF, 0x7102]);
    cpu.v[15] = 7;
    step(&mut cpu, 2);
    assert_eq!((cpu.v[1], cpu.v[15]), (1, 7));
}

#[test]
fn register_transfers_include_last_register_and_preserve_index() {
    for x in [0, 1, 15] {
        let mut cpu = machine(&[0xF055 | (x << 8), 0xF065 | (x << 8)]);
        cpu.i = (MEMORY_SIZE - x as usize - 1) as u16;
        let index = cpu.i;
        for register in 0..=x as usize {
            cpu.v[register] = register as u8 + 17;
        }
        step(&mut cpu, 1);
        assert_eq!(cpu.memory[MEMORY_SIZE - 1], x as u8 + 17);
        cpu.v = [0; 16];
        step(&mut cpu, 1);
        assert_eq!(cpu.v[x as usize], x as u8 + 17);
        assert_eq!(cpu.i, index);
    }
}

#[test]
fn drawing_wraps_and_reports_xor_collision() {
    let mut cpu = machine(&[0xD122, 0xD122, 0x00E0]);
    cpu.i = 0x300;
    cpu.memory[0x300..0x302].copy_from_slice(&[0xC0, 0x80]);
    cpu.v[1] = 63;
    cpu.v[2] = 31;
    assert!(cpu.step([false; 16]).unwrap().screen_changed);
    assert!(cpu.screen()[31][63] && cpu.screen()[31][0] && cpu.screen()[0][63]);
    assert_eq!(cpu.v[15], 0);
    step(&mut cpu, 1);
    assert_eq!(cpu.v[15], 1);
    assert!(cpu.screen().iter().flatten().all(|pixel| !pixel));
    cpu.screen_buffer[0][0] = true;
    assert!(cpu.step([false; 16]).unwrap().screen_changed);
    assert!(!cpu.screen()[0][0]);
}

#[test]
fn calls_return_to_following_instruction_and_skips_advance_four_bytes() {
    let mut cpu = machine(&[0x2208, 0x3107, 0x61FF, 0x120C, 0x6107, 0x00EE, 0x6209]);
    step(&mut cpu, 6);
    assert_eq!((cpu.v[1], cpu.v[2], cpu.program_counter), (7, 9, 0x20E));
    assert!(cpu.stack.is_empty());
}

#[test]
fn key_wait_pauses_instructions_but_not_timers() {
    let mut cpu = machine(&[0xF10A, 0x6209]);
    cpu.delay_timer = 2;
    cpu.sound_timer = 1;
    step(&mut cpu, 3);
    assert_eq!(cpu.v[2], 0);
    assert_eq!(cpu.delay_timer, 2);
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 1);
    assert!(!cpu.sound_active());
    let mut keys = [false; 16];
    keys[5] = true;
    keys[9] = true;
    cpu.step(keys).unwrap();
    assert_eq!(cpu.v[1], 5);
    step(&mut cpu, 1);
    assert_eq!(cpu.v[2], 9);
    cpu.tick_timers();
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 0);
}

#[test]
fn key_skips_use_low_nibble() {
    let mut cpu = machine(&[0xE19E, 0xE1A1]);
    cpu.v[1] = 0xFF;
    step(&mut cpu, 1);
    assert_eq!(cpu.program_counter, 0x202);
    step(&mut cpu, 1);
    assert_eq!(cpu.program_counter, 0x206);
}

#[test]
fn bcd_and_font_addressing() {
    let mut cpu = machine(&[0xF133, 0xF129]);
    cpu.i = 0x300;
    cpu.v[1] = 254;
    step(&mut cpu, 1);
    assert_eq!(&cpu.memory[0x300..0x303], &[2, 5, 4]);
    cpu.v[1] = 0xA;
    step(&mut cpu, 1);
    assert_eq!(&cpu.memory[cpu.i as usize..cpu.i as usize + 5], &DIGIT_A);
}

#[test]
fn loading_validates_size_and_resets_state() {
    let mut cpu = machine(&[0x61FF]);
    step(&mut cpu, 1);
    assert!(cpu.load(&[]).is_err());
    assert!(cpu.load(&vec![0; MAX_ROM_SIZE + 1]).is_err());
    cpu.load(&vec![0xAA; MAX_ROM_SIZE]).unwrap();
    assert_eq!(cpu.memory[4095], 0xAA);
    assert_eq!(cpu.program_counter, 0x200);
    assert_eq!(cpu.v, [0; 16]);
}

#[test]
fn invalid_programs_return_errors_instead_of_panicking() {
    for op in [0x00EE, 0xFFFF, 0x5121, 0x8128, 0xE100] {
        assert!(machine(&[op]).step([false; 16]).is_err(), "{op:04X}");
    }
    for op in [0xF133, 0xF155, 0xF165, 0xD122] {
        let mut cpu = machine(&[op]);
        cpu.i = 4095;
        assert!(cpu.step([false; 16]).is_err(), "{op:04X}");
    }
    let mut cpu = machine(&[0x1FFF]);
    step(&mut cpu, 1);
    assert!(cpu.step([false; 16]).is_err());
    let mut cpu = machine(&[0x2200]);
    step(&mut cpu, 16);
    assert!(cpu.step([false; 16]).is_err());
}

#[test]
fn bundled_ibm_logo_draws_a_stable_frame() {
    let mut cpu = Chip8::new();
    cpu.load(include_bytes!("../../roms/programs/IBM Logo.ch8"))
        .unwrap();
    step(&mut cpu, 100);
    let frame = *cpu.screen();
    let lit = frame.iter().flatten().filter(|pixel| **pixel).count();
    assert_eq!(lit, 208);
    step(&mut cpu, 100);
    assert_eq!(cpu.screen(), &frame);
}
