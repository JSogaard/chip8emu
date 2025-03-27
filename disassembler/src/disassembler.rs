use crate::errors::Error;

fn disassembler(rom_path: &str) -> Result<(), Error> {
    let rom = std::fs::read(rom_path).map_err(|_| Error::FileReadError)?;
    let assembly: Vec<String> = Vec::new();

    for (i, bytes) in rom.chunks_exact(2).enumerate() {
        let opcode = ((bytes[0] as u32) << 8) | bytes[2] as u32;
        let address = 0x200 + i as u32;

        let line = match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => clear_display(address),
                0x00EE => return_subroutine(address),
                _ => todo!()
            },

            0x1000 => jump(address, opcode),
            0x2000 => call_subroutine(address, opcode),
            0x3000 => skip_equal(address, opcode),
            0x4000 => skip_not_equal(address, opcode),
            0x5000 => skip_register_equal(address, opcode),
            0x6000 => load_number(address, opcode),
            0x7000 => add_number(address, opcode),

            // Register loading opcodes
            0x8000 => match opcode & 0x000F {
                0x0 => move_register(address, opcode),
                // OR
                0x1 => load_register_op(address, opcode, "OR"),
                // AND
                0x2 => load_register_op(address, opcode, "AND"),
                // XOR
                0x3 => load_register_op(address, opcode, "XOR"),
                0x4 => add_register_carry(address, opcode),
                0x5 => sub_register(address, opcode),
                0x6 => shift_right(address, opcode),
                0x7 => sub_register_reversed(address, opcode),
                0xE => shift_left(address, opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0x9000 => skip_register_not_equal(address, opcode),
            0xA000 => load_i(address, opcode),
            0xB000 => jump_plus(address, opcode),
            0xC000 => random_and(opcode),
            0xD000 => draw_sprite(opcode, display),

            0xE000 => match opcode & 0x00FF {
                0x9E => skip_if_keypress(opcode, input),
                0xA1 => skip_if_not_keypress(opcode, input),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0xF000 => match opcode & 0x00FF {
                0x07 => load_delay_timer(opcode),
                0x0A => wait_for_keypress(opcode, input),
                0x15 => set_delay_timer(opcode),
                0x18 => set_sound_timer(opcode),
                0x1E => load_add_i(opcode),
                0x29 => find_character(opcode),
                0x33 => store_bcd(opcode),
                0x55 => dump_registers_to_ram(opcode)
                0x65 => load_registers_from_ram(opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            }

            _ => return Err(Error::UnknownOpcodeError(opcode)),
        };
    }

    Ok(())
}

fn opcode_no_arg(address: u32, mnemonic: &str) -> String {
    format!("{address:03X}: {mnemonic}")
}

fn opcode_one_arg(address: u32, mnemonic: &str, arg: u32) -> String {
    format!("{address:03X}: {mnemonic} {arg:#X}")
}

fn opcode_reg_arg(address: u32, mnemonic: &str, reg: u32, arg: u32) -> String {
    format!("{address:03X}: {mnemonic} V{reg:X}, {arg:#X}")
}

fn opcode_reg_reg(address: u32, mnemonic: &str, reg1: u32, reg2: u32) -> String {
    format!("{address:03X}: {mnemonic} V{reg1:X}, V{reg2:X}")
}

/***** OPCODE FUNCTIONS *****/

fn clear_display(address: u32) -> String {
    opcode_no_arg(address, "CLS")
}

fn return_subroutine(address: u32) -> String {
    opcode_no_arg(address, "RTS")
}

fn sys_call(address: u32, target: u32) -> String {
    opcode_one_arg(address, "SYS", target)
}

fn jump(address: u32, opcode: u32) -> String {
    let target = opcode & 0x0FFF;
    opcode_one_arg(address, "JUMP", target)
}

fn call_subroutine(address: u32, opcode: u32) -> String {
    let target = opcode & 0x0FFF;
    opcode_one_arg(address, "CALL", target)
}

fn skip_equal(address: u32, opcode: u32) -> String {
    let reg = (opcode & 0x0F00) >> 8;
    let number = opcode & 0x00FF;
    opcode_reg_arg(address, "SKE", reg, number)
}

fn skip_not_equal(address: u32, opcode: u32) -> String {
    let reg = (opcode & 0x0F00) >> 8;
    let number = opcode & 0x00FF;
    opcode_reg_arg(address, "SKNE", reg, number)
}

fn skip_register_equal(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_arg(address, "SKRE", reg_x, reg_y)
}

fn load_number(address: u32, opcode: u32) -> String {
    let reg = (opcode & 0x0F00) >> 8;
    let number = opcode & 0x00FF;
    opcode_reg_arg(address, "LOAD", reg, number)
}

fn add_number(address: u32, opcode: u32) -> String {
    let reg = (opcode & 0x0F00) >> 8;
    let number = opcode & 0x00FF;
    opcode_reg_arg(address, "ADD", reg, number)
}

fn move_register(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "MOVE", reg_x, reg_y)
}

fn load_register_op(address: u32, opcode: u32, operation: &str) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, operation, reg_x, reg_y)
}

fn add_register_carry(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "ADDR", reg_x, reg_y)
}

fn sub_register(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "SUB", reg_x, reg_y)
}

fn shift_right(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "SHR", reg_x, reg_y)
}

fn sub_register_reversed(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "SUBR", reg_x, reg_y)
}

fn shift_left(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "SHL", reg_x, reg_y)
}

fn skip_register_not_equal(address: u32, opcode: u32) -> String {
    let reg_x = (opcode & 0x0F00) >> 8;
    let reg_y = (opcode & 0x00F0) >> 4;
    opcode_reg_reg(address, "SKRNE", reg_x, reg_y)
}

fn load_i(address: u32, opcode: u32) -> String {
    let number = opcode & 0x0FFF;
    opcode_one_arg(address, "LOADI", number)
}

fn jump_plus(address: u32, opcode: u32) -> String {
    let number = opcode & 0x0FFF;
    opcode_one_arg(number, "JUMPI", address)
}