use rand::Rng;

use crate::errors::Error;
use crate::errors::Result;

pub const RAM_SIZE: usize = 4096;
pub const NUM_REGS: usize = 16;
pub const CARRY_REGISTER: usize = NUM_REGS - 1;
pub const STACK_SIZE: usize = 16;
pub const START_ADDR: u16 = 0x200;
pub const MAX_ROM_SIZE: usize = RAM_SIZE - START_ADDR as usize;

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
    // Program counter
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    // General purpose registers
    v_reg: [u8; NUM_REGS],
    // Index register
    i_reg: u16,
    // Stack pointer
    sp: u16,
    stack: [u16; STACK_SIZE],
    // Sound timer
    st: u8,
    // Delay timer
    dt: u8,
    redraw_flag: bool,
    rom_loaded: bool,
    rng: rand::rngs::ThreadRng,
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
            rom_loaded: false,
            rng: rand::rng(),
        };

        // Copying font set into ram from address 0x50 (80)
        // Get target location in RAM as slice and copy font set to it
        emulator.ram[FONTSET_ADDR as usize..(FONTSET_ADDR as usize + FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        emulator
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        if rom.len() <= MAX_ROM_SIZE {
            self.ram[START_ADDR as usize..].copy_from_slice(rom);
        } else {
            return Err(Error::InvalidRomSizeError);
        }

        Ok(())
    }

    pub fn redraw_flag(&self) -> bool {
        self.redraw_flag
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.st = 0;
        self.dt = 0;
        self.redraw_flag = false;
    }

    pub fn cycle(&mut self) -> Result<bool> {
        // Returns bool draw flag

        // Check if the end of RAM is reached
        if self.pc as usize >= RAM_SIZE {
            return Err(Error::InvalidRamAddressError);
        }

        self.redraw_flag = false;
        // Get opcode as u16
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[(self.pc + 1) as usize] as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.pc += 2;

        // DECODE AND EXECUTE OPCODE
        // Filter op code to match only the first half byte
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.clear_screen(),
                0x00EE => self.return_subroutine()?,
                // If op code is 0NNN - call machine code subroutine,
                // which isn't implemented.
                _ => {
                    return Err(Error::InvalidOpcodeError(
                        "0NNN - Call machine code routine".into(),
                    ))
                }
            },

            0x1000 => self.jump(opcode),
            0x2000 => self.call_subroutine(opcode)?,
            0x3000 => self.skip_equal(opcode),
            0x4000 => self.skip_not_equal(opcode),
            0x5000 => self.skip_register_equal(opcode),
            0x6000 => self.load_number(opcode),
            0x7000 => self.add_number(opcode),

            // Register loading op codes
            0x8000 => match opcode & 0x000F {
                0x0 => self.load_register_op(opcode, |_, vy| vy),
                0x1 => self.load_register_op(opcode, |vx, vy| vx | vy),
                0x2 => self.load_register_op(opcode, |vx, vy| vx & vy),
                0x3 => self.load_register_op(opcode, |vx, vy| vx ^ vy),
                0x4 => self.add_register_carry(opcode),
                0x5 => self.sub_register(opcode),
                0x6 => self.shift_right(opcode),
                0x7 => self.sub_register(opcode),
                0xE => self.shift_left(opcode),
                _ => return Err(Error::UnknownOpcodeError(opcode)),
            },

            0x9000 => self.skip_register_not_equal(opcode),
            0xA000 => self.load_i(opcode),
            0xB000 => self.jump_plus(opcode),
            0xC000 => self.random_and(opcode),

            _ => return Err(Error::UnknownOpcodeError(opcode)),
        }

        todo!()
    }

    fn push(&mut self, val: u16) -> Result<()> {
        // Check for stack overflow
        if self.sp >= STACK_SIZE as u16 {
            return Err(Error::StackOverflowError);
        }

        self.stack[self.sp as usize] = val;
        self.sp += 1;

        Ok(())
    }

    fn pop(&mut self) -> Result<u16> {
        if self.sp == 0 {
            return Err(Error::StackUnderflowError);
        }

        self.sp -= 1;
        Ok(self.stack[self.sp as usize])
    }

    //************************************************************//
    //                      OPCODE METHODS                        //
    //************************************************************//

    /// Opcode 00E0
    /// Clear screen
    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.redraw_flag = true;
    }

    /// Opcode 00EE
    /// Return from subroutine
    fn return_subroutine(&mut self) -> Result<()> {
        // Pop return address from stack and set PC to it
        let return_address = self.pop()?;
        self.pc = return_address;

        Ok(())
    }

    /// Opcode 1NNN
    /// Jump to address NNN
    fn jump(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// Opcode 2NNN
    /// Call subroutine at address NNN
    fn call_subroutine(&mut self, opcode: u16) -> Result<()> {
        // PC is pushed to stack to remember where to return after subroutine
        self.push(self.pc)?;
        let address = opcode & 0x0FFF;
        self.pc = address;

        Ok(())
    }

    /// Opcode 3XNN
    /// Skip next instruction if VX == NN
    fn skip_equal(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        if number == self.v_reg[register] {
            self.pc += 2;
        }
    }

    /// Opcode 4XNN
    /// Skip next instruction if VX != NN
    fn skip_not_equal(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        if number != self.v_reg[register] {
            self.pc += 2;
        }
    }

    /// Opcode 5XY0
    /// Skip next instruction if VX == VY
    fn skip_register_equal(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        if self.v_reg[reg_x] == self.v_reg[reg_y] {
            self.pc += 2;
        }
    }

    /// Opcode 6XNN
    /// Load NN into VX
    fn load_number(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        self.v_reg[register] = number;
    }

    /// Opcode 7XNN
    /// Add NN to VX (VX += NN)
    fn add_number(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        self.v_reg[register] = self.v_reg[register].wrapping_add(number);
    }

    /// Opcode 8XY1 to 8XY3
    /// Load op(VX, VY) into VX
    fn load_register_op<F: Fn(u8, u8) -> u8>(&mut self, opcode: u16, op: F) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let value = op(self.v_reg[reg_x], self.v_reg[reg_y]);
        self.v_reg[reg_x] = value;
    }

    /// Opcode 8XY4
    /// Add value of VY to VX (VX += VY) and enable carry register if overflowing
    fn add_register_carry(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let result = self.v_reg[reg_x].wrapping_add(self.v_reg[reg_y]);
        self.v_reg[reg_x] = result;

        // Enable carry register if addition overflows
        self.v_reg[CARRY_REGISTER] = (result < self.v_reg[reg_x]) as u8;
    }

    /// Opcode 8XY5
    /// Subtract value of VY from VX (VX -= VY) and enable carry register
    /// if not borrowing
    fn sub_register(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);

        // Enable carry register if subtraction borrows
        let not_borrow = (self.v_reg[reg_x] > self.v_reg[reg_y]) as u8;
        self.v_reg[CARRY_REGISTER] = not_borrow;

        let result = self.v_reg[reg_x].wrapping_sub(self.v_reg[reg_y]);
        self.v_reg[reg_x] = result;
    }

    /// Opcode 8XY6
    /// Set carry register to least significant bit of VX
    /// and shift VX one bit right
    fn shift_right(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        self.v_reg[CARRY_REGISTER] = self.v_reg[register] & 0x1;
        self.v_reg[register] >>= 1;
    }

    /// Opcode 8XY7
    /// Subtract the value of VX from VY and load result into VX (VX = VY - VX)
    /// then enable carry register if not borrowing
    fn sub_register_reversed(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);

        // Enable carry register if subtraction borrows
        let not_borrow = (self.v_reg[reg_y] > self.v_reg[reg_x]) as u8;
        self.v_reg[CARRY_REGISTER] = not_borrow;

        let result = self.v_reg[reg_y].wrapping_sub(self.v_reg[reg_x]);
        self.v_reg[reg_x] = result;
    }

    /// Opcode 8XYE
    /// Set carry register to least significant bit of VX
    /// and shift VX one bit left
    fn shift_left(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        self.v_reg[CARRY_REGISTER] = self.v_reg[register] & 0x1;
        self.v_reg[register] <<= 1;
    }

    /// Opcode 9XY0
    /// Skip next instruction if VX != VY
    fn skip_register_not_equal(&mut self, opcode: u16) {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        if self.v_reg[reg_x] != self.v_reg[reg_y] {
            self.pc += 2;
        }
    }

    /// Opcode ANNN
    /// Set I register to NNN
    fn load_i(&mut self, opcode: u16) {
        self.i_reg = opcode & 0x0FFF;
    }

    /// Opcode BNNN
    /// Jump to address at V0 + NNN
    fn jump_plus(&mut self, opcode: u16) {
        self.pc = self.v_reg[0] as u16 + (opcode & 0x0FFF);
    }

    /// Opcode CXNN
    /// Generate random number, R, from 0 to 255 and add R AND NN to VX
    fn random_and(&mut self, opcode: u16) {
        let register = ((opcode & 0x0F00) >> 8) as usize;
        let number = (opcode & 0x00FF) as u8;
        let random: u8 = self.rng.random();
        self.v_reg[register] = random & number;
    }
}

// HELPER FUNCTIONS

fn decode_middle_registers(opcode: u16) -> (usize, usize) {
    let reg_x = ((opcode & 0x0F00) >> 8) as usize;
    let reg_y = ((opcode & 0x00F0) >> 4) as usize;
    (reg_x, reg_y)
}
