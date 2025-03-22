use sdl2::keyboard::Keycode;

/// Get the register numbers X and Y in an opcode: NXYN
pub fn decode_middle_registers(opcode: u16) -> (u16, u16) {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    (reg_x, reg_y)
}

/// Get the nth bit in a byte as a boolean starting
/// with most significant bit and zero-based indexing
pub fn bit_to_bool(byte: u8, n: u8) -> bool {
    ((byte >> (7 - n)) & 0x1) != 0
}

/// Matches SDL keycode to corresponding key number
pub fn keycode_to_button(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::Num1 => Some(0x0),
        Keycode::Num2 => Some(0x1),
        Keycode::Num3 => Some(0x2),
        Keycode::Num4 => Some(0x3),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0x7),
        Keycode::A => Some(0x8),
        Keycode::S => Some(0x9),
        Keycode::D => Some(0xA),
        Keycode::F => Some(0xB),
        Keycode::Z => Some(0xC),
        Keycode::X => Some(0xD),
        Keycode::C => Some(0xE),
        Keycode::V => Some(0xF),
        _ => None,
    }
}