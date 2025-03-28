use std::{fs::File, io::Write};

use crate::errors::Error;

pub fn disassembler(rom_path: &str, output: Option<String>) -> Result<(), Error> {
    let rom = std::fs::read(rom_path).map_err(|e| Error::FileReadError(e.to_string()))?;
    let mut assembly: Vec<String> = Vec::new();

    for (i, bytes) in rom.chunks_exact(2).enumerate() {
        let opcode = ((bytes[0] as u32) << 8) | bytes[1] as u32;
        let address = 0x200 + i as u32;

        let line = match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => clear_display(address),
                0x00EE => return_subroutine(address),
                _ => sys_call(address, opcode),
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
            0xC000 => random_and(address, opcode),
            0xD000 => draw_sprite(address, opcode),

            0xE000 => match opcode & 0x00FF {
                0x9E => skip_if_keypress(address, opcode),
                0xA1 => skip_if_not_keypress(address, opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0xF000 => match opcode & 0x00FF {
                0x07 => move_delay_timer(address, opcode),
                0x0A => wait_for_keypress(address, opcode),
                0x15 => set_delay_timer(address, opcode),
                0x18 => set_sound_timer(address, opcode),
                0x1E => load_add_i(address, opcode),
                0x29 => find_character(address, opcode),
                0x33 => store_bcd(address, opcode),
                0x55 => dump_registers_to_ram(address, opcode),
                0x65 => load_registers_from_ram(address, opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            _ => return Err(Error::UnknownOpcodeError(opcode)),
        };
        
        // println!("{opcode:04X},   {:03X}       {line}", i * 2);
        
        assembly.push(line);

    }

    let assembly = assembly.join("\n");
    match output {
        Some(output) => {
            let mut file =
                File::create_new(output).map_err(|e| Error::FileWriteError(e.to_string()))?;
            file.write_all(assembly.as_bytes())
                .map_err(|e| Error::FileWriteError(e.to_string()))?;
        }
        None => println!("{}", assembly),
    }

    Ok(())
}

fn format_no_arg(address: u32, mnemonic: &str) -> String {
    format!("{address:03X}: {mnemonic:<6}")
}

fn format_one_arg(address: u32, mnemonic: &str, arg: u32) -> String {
    format!("{address:03X}: {mnemonic:<6} {arg:#X}")
}

fn format_one_reg(address: u32, mnemonic: &str, reg: u32) -> String {
    format!("{address:03X}: {mnemonic:<6} V{reg:X}")
}

fn format_reg_arg(address: u32, mnemonic: &str, reg: u32, arg: u32) -> String {
    format!("{address:03X}: {mnemonic:<6} V{reg:X}, {arg:#X}")
}

fn format_reg_reg(address: u32, mnemonic: &str, reg1: u32, reg2: u32) -> String {
    format!("{address:03X}: {mnemonic:<6} V{reg1:X}, V{reg2:X}")
}

fn format_reg_reg_arg(address: u32, mnemonic: &str, reg1: u32, reg2: u32, arg: u32) -> String {
    format!("{address:03X}: {mnemonic:<6} V{reg1:X}, V{reg2:X}, {arg:#X}")
}

fn get_hex_digit(hex: u32, i: u32) -> u32 {
    (hex >> (i * 4)) & 0xF
}

/***** OPCODE FUNCTIONS *****/

fn clear_display(address: u32) -> String {
    format_no_arg(address, "CLS")
}

fn return_subroutine(address: u32) -> String {
    format_no_arg(address, "RTS")
}

fn sys_call(address: u32, opcode: u32) -> String {
    let target = opcode & 0x0FFF;
    format_one_arg(address, "SYS", target)
}

fn jump(address: u32, opcode: u32) -> String {
    let target = opcode & 0x0FFF;
    format_one_arg(address, "JUMP", target)
}

fn call_subroutine(address: u32, opcode: u32) -> String {
    let target = opcode & 0x0FFF;
    format_one_arg(address, "CALL", target)
}

fn skip_equal(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    let number = opcode & 0x00FF;
    format_reg_arg(address, "SKE", reg, number)
}

fn skip_not_equal(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    let number = opcode & 0x00FF;
    format_reg_arg(address, "SKNE", reg, number)
}

fn skip_register_equal(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_arg(address, "SKRE", reg_x, reg_y)
}

fn load_number(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    let number = opcode & 0x00FF;
    format_reg_arg(address, "LOAD", reg, number)
}

fn add_number(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    let number = opcode & 0x00FF;
    format_reg_arg(address, "ADD", reg, number)
}

fn move_register(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "MOVE", reg_x, reg_y)
}

fn load_register_op(address: u32, opcode: u32, operation: &str) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, operation, reg_x, reg_y)
}

fn add_register_carry(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "ADDR", reg_x, reg_y)
}

fn sub_register(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "SUB", reg_x, reg_y)
}

fn shift_right(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "SHR", reg_x, reg_y)
}

fn sub_register_reversed(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "SUBR", reg_x, reg_y)
}

fn shift_left(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "SHL", reg_x, reg_y)
}

fn skip_register_not_equal(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    format_reg_reg(address, "SKRNE", reg_x, reg_y)
}

fn load_i(address: u32, opcode: u32) -> String {
    let number = opcode & 0x0FFF;
    format_one_arg(address, "LOADI", number)
}

fn jump_plus(address: u32, opcode: u32) -> String {
    let number = opcode & 0x0FFF;
    format_one_arg(number, "JUMPI", address)
}

fn random_and(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    let number = opcode & 0x00FF;
    format_reg_arg(address, "RAND", reg, number)
}

fn draw_sprite(address: u32, opcode: u32) -> String {
    let reg_x = get_hex_digit(opcode, 2);
    let reg_y = get_hex_digit(opcode, 1);
    let number = opcode & 0x000F;
    format_reg_reg_arg(address, "DRAW", reg_x, reg_y, number)
}

fn skip_if_keypress(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "SKEYD", reg)
}

fn skip_if_not_keypress(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "SKEYU", reg)
}

fn move_delay_timer(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "MOVEDT", reg)
}

fn wait_for_keypress(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "KEYW", reg)
}

fn set_delay_timer(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "LOADD", reg)
}

fn set_sound_timer(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "LOADS", reg)
}

fn load_add_i(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "ADDI", reg)
}

fn find_character(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "LDCHR", reg)
}

fn store_bcd(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "BCDI", reg)
}

fn dump_registers_to_ram(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "STORE", reg)
}

fn load_registers_from_ram(address: u32, opcode: u32) -> String {
    let reg = get_hex_digit(opcode, 2);
    format_one_reg(address, "READ", reg)
}
