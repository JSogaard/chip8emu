use crate::errors::*;

pub const RAM_SIZE: usize = 4096;
pub const NUM_REGS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const START_ADDR: u16 = 0x200;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const FONTSET_SIZE: usize = 16 * 5;
pub const FONTSET_ADDR: u16 = 0x050;
pub const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    st: u8,
    dt: u8,
    redraw_flag: bool,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emulator = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            st: 0,
            dt: 0,
            redraw_flag: false,
        };

        // Copying font set into ram from address 0x50 (80)
        // Get target location in RAM as slice and copy font set to it
        emulator.ram[FONTSET_ADDR as usize..(FONTSET_ADDR as usize + FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        emulator
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram[START_ADDR as usize..].copy_from_slice(rom);
    }

    pub fn cycle(&mut self) -> Result<bool> {
        // Returns bool draw flag

        // Get opcode as u16
        let low_byte = self.ram[self.pc as usize] as u16;
        let high_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op_code = (low_byte << 8) | high_byte;
        self.pc += 2;

        // Filter op code to match only the first half byte
        match op_code & 0xF000 {
            0x0000 => match op_code {
                0x00E0 => self.clear_screen(),
                0x00EE => self.return_subroutine(),

                // If op code is 0x0NNN - call machine code subroutine,
                // which isn't implemented.
                _ => {
                    return Err(Error::InvalidOpcodeError(
                        "Call machine code routine".into(),
                    ))
                }
            },

            0x1000 => self.jump(op_code),
            0x2000 => self.call_subroutine(op_code),
            0x3000 => self.skip_equal(op_code),
            0x4000 => self.skip_not_equal(op_code),
            0x5000 => self.skip_register_equal(op_code),
            0x6000 => self.set_register_to_number(op_code),

            _ => {}
        }

        todo!()
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    //************************************************************//
    //                      OPCODE METHODS                        //
    //************************************************************//


    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    fn return_subroutine(&mut self) {
        // Pop return address from stack and set PC to it
        let return_address = self.pop();
        self.pc = return_address;
    }

    fn jump(&mut self, op_code: u16) {
        let address = op_code & 0x0FFF;
        self.pc = address;
    }

    fn call_subroutine(&mut self, op_code: u16) {
        // PC is pushed to stack to remember where to return after subroutine
        self.push(self.pc);
        let address = op_code & 0x0FFF;
        self.pc = address;
    }

    fn skip_equal(&mut self, op_code: u16) {
        let register = (op_code & 0x0F00 >> 8) as usize;
        let number = (op_code & 0x00FF) as u8;
        if number == self.v_reg[register] {
            self.pc += 2;
        }
    }

    fn skip_not_equal(&mut self, op_code: u16) {
        let register = (op_code & 0x0F00 >> 8) as usize;
        let number = (op_code & 0x00FF) as u8;
        if number != self.v_reg[register] {
            self.pc += 2;
        }
    }

    fn skip_register_equal(&mut self, op_code: u16) {
        let register_x = (op_code & 0x0F00 >> 8) as usize;
        let register_y = (op_code & 0x00F0 >> 4) as usize;
        if self.v_reg[register_x] == self.v_reg[register_y] {
            self.pc += 2;
        }
    }

    fn set_register_to_number(&mut self, op_code: u16) {
        let register = (op_code & 0x0F00 >> 8) as usize;
        let number = (op_code & 0x00FF) as u8;
        self.v_reg[register] = number;
    }
}
