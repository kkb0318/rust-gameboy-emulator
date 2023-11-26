use super::Cpu;
use crate::peripherals::Peripherals;

impl Cpu {
    /// no operation  何もせず次の命令をfetchするだけ
    pub fn nop(&mut self, bus: &mut Peripherals) {
        self.fetch(bus)
    }
}

macro_rules! step {
  ($d:expr, {$($c:tt : $e:expr,)*}) => {
    static STEP: AtomicU8 = AtomicU8::new(0);
    #[allow(dead_code)]
    static VAL8: AtomicU8 = AtomicU8::new(0);
    #[allow(dead_code)]
    static VAL16: AtomicU16 = AtomicU16::new(0);
    $(if STEP.load(Relaxed) == $c { $e })* else { return $d; }
  };
}
pub(crate) use step;
macro_rules! go {
    ($e:expr) => {
        STEP.store($e, Relaxed)
    };
}
pub(crate) use go;
