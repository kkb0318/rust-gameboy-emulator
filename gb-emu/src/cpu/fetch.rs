use crate::cpu::Cpu;
use crate::peripherals::Peripherals;

impl Cpu {
    /// プログラムカウンタが示すアドレスに格納された命令（8 bit）をbus から読み出し，
    /// プログラムカウンタを1 インクリメントする. これにより次のfetchでは1つ後ろの命令が読み出される
    pub fn fetch(&mut self, bus: &Peripherals) {
        self.ctx.opcode = bus.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        self.ctx.cb = false;
    }
}
