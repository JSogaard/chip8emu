use rand::Rng;

use crate::errors::{Error, Result};
use crate::helpers::decode_middle_registers;
use crate::memory::{Memory, RAM_SIZE, START_ADDR};
use crate::screen::{Screen, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::stack::Stack;

pub const NUM_REGS: usize = 16;
pub const CARRY_REGISTER: usize = NUM_REGS - 1;

pub struct Cpu {
    // Program counter
    pc: u16,
    ram: Memory,
    screen: Screen,
    // General purpose registers
    v_reg: [u8; NUM_REGS],
    // Index register
    i_reg: u16,
    stack: Stack,
    // Sound timer
    st: u8,
    // Delay timer
    dt: u8,
    redraw_flag: bool,
    rng: rand::rngs::ThreadRng,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: Memory::new(),
            screen: Screen::new(),
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: Stack::new(),
            st: 0,
            dt: 0,
            redraw_flag: false,
            rng: rand::rng(),
        }
    }

    pub fn redraw_flag(&self) -> bool {
        self.redraw_flag
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram.reset();
        self.screen.reset();
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack.reset();
        self.st = 0;
        self.dt = 0;
        self.redraw_flag = false;
    }

    pub fn cycle(&mut self) -> Result<()> {
        // Check if ROM as been loaded into RAM
        if !self.ram.rom_loaded() {
            return Err(Error::MissingRomError);
        }

        // Check if the end of RAM is reached
        if self.pc as usize >= RAM_SIZE {
            return Err(Error::InvalidRamAddressError);
        }

        self.redraw_flag = false;
        // Get opcode as u16
        let high_byte = self.ram.read(self.pc) as u16;
        let low_byte = self.ram.read(self.pc + 1) as u16;
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

    //************************************************************//
    //                      OPCODE METHODS                        //
    //************************************************************//

    /// Opcode 00E0
    /// Clear screen
    fn clear_screen(&mut self) {
        self.screen.reset();
        self.redraw_flag = true;
    }

    /// Opcode 00EE
    /// Return from subroutine
    fn return_subroutine(&mut self) -> Result<()> {
        // Pop return address from stack and set PC to it
        let return_address = self.stack.pop()?;
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
        self.stack.push(self.pc)?;
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

    /// Opcode DXYN
    /// Draws N-byte (heigh of N pixels) on screen and enables
    /// carry register if there is collision
    fn draw_sprite(&mut self, opcode: u16) -> Result<()> {
        let (reg_x, reg_y) = decode_middle_registers(opcode);
        let rows = opcode & 0x000F;

        // Check if sprite bounds are within valid RAM addresses
        if self.i_reg + rows > RAM_SIZE as u16 {
            return Err(Error::InvalidRamAddressError);
        }

        // Set x and y coords to VX and VY with wrapping for the starting coord
        let x_coord = self.v_reg[reg_x] % SCREEN_WIDTH as u8;
        let y_coord = self.v_reg[reg_y] % SCREEN_HEIGHT as u8;
        self.v_reg[CARRY_REGISTER] = 0;

        let sprite = self.ram.read_slice(self.i_reg, rows);

        // Call Screen.draw()
        self.screen.draw(sprite, x_coord, y_coord);
        self.redraw_flag = true;

        Ok(())
    }
}
