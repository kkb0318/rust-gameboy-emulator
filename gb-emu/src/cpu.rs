use crate::cpu::registers::Registers;
use crate::peripherals::Peripherals;

mod decode;
mod fetch;
mod instructions;
mod registers;

#[derive(Default)]
struct Ctx {
    opcode: u8,
    cb: bool,
}
pub struct Cpu {
    regs: Registers,
    ctx: Ctx,
}

impl Cpu {
    pub fn emulate_cycle(&mut self, bus: &mut Peripherals) {
        self.decode(bus);
    }
}
