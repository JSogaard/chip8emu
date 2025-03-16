use crate::errors::{Error, Result};

pub const STACK_SIZE: usize = 16;

#[derive(Debug)]
pub struct Stack {
    stack: [u16; STACK_SIZE],
    // Stack pointer
    sp: u16,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }

    /// Push return address to stack
    pub fn push(&mut self, val: u16) -> Result<()> {
        // Check for stack overflow
        if self.sp >= STACK_SIZE as u16 {
            return Err(Error::StackOverflowError);
        }

        self.stack[self.sp as usize] = val;
        self.sp += 1;

        Ok(())
    }

    /// Pop return address from stack
    pub fn pop(&mut self) -> Result<u16> {
        if self.sp == 0 {
            return Err(Error::StackUnderflowError);
        }

        self.sp -= 1;
        Ok(self.stack[self.sp as usize])
    }

    pub fn reset(&mut self) {
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
    }
}