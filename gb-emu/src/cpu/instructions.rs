use super::Cpu;
use crate::peripherals::Peripherals;

impl Cpu {
    /// no operation  何もせず次の命令をfetchするだけ
    pub fn nop(&mut self, bus: &mut Peripherals) {
        self.fetch(bus)
    }
}
