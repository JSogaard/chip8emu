/// Get the register numbers X and Y in an opcode: NXYN
pub fn decode_middle_registers(opcode: u16) -> (usize, usize) {
    let reg_x = ((opcode & 0x0F00) >> 8) as usize;
    let reg_y = ((opcode & 0x00F0) >> 4) as usize;
    (reg_x, reg_y)
}

/// Get the nth bit in a byte as a boolean starting
/// with most significant bit and zero-based indexing
 pub fn bit_to_bool(byte: u8, n: u8) -> bool {
    ((byte >> (7 - n)) & 0x1) != 0
}