use crate::peripherals::Peripherals;

use super::operand::{Cond, Imm8, Reg8, IO8};
use super::Cpu;

impl Cpu {
    pub fn decode(&mut self, bus: &mut Peripherals) {
        if self.ctx.cb {
            // 2つ目の表の命令の実行中である場合
            self.cb_decode(bus);
            return;
        }
        match self.ctx.opcode {
            0x00 => self.nop(bus),
            0x20 => self.jr_c(bus, Cond::NZ), // 例
            0xCB => self.cb_prefixed(bus),
            _ => panic!("Not implemented: {:02x}", self.ctx.opcode),
        }
    }
    pub fn cb_decode(&mut self, bus: &mut Peripherals) {
        match self.ctx.opcode {
            // ここに2つ目の表の命令を書く
            0x10 => self.rl(bus, Reg8::B), // 例
            _ => panic!("Not implemented: {:02x}", self.ctx.opcode),
        }
    }
    pub fn cb_prefixed(&mut self, bus: &mut Peripherals) {
        if let Some(v) = self.read8(bus, Imm8) {
            self.ctx.opcode = v; // 2つ目の表のオペコード
            self.ctx.cb = true; // 2つ目の表の命令を実行中であることを覚えておく
            self.cb_decode(bus);
        }
    }
}
