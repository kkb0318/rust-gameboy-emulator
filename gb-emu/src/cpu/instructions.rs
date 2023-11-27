use super::operand::{IO16, IO8};
use super::Cpu;
use crate::peripherals::Peripherals;
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering::Relaxed};

impl Cpu {
    /// no operation  何もせず次の命令をfetchするだけ
    pub fn nop(&mut self, bus: &mut Peripherals) {
        self.fetch(bus)
    }

    // sの値をdに格納する (8 bit)
    pub fn ld<D: Copy, S: Copy>(&mut self, bus: &mut Peripherals, dst: D, src: S)
    where
        Self: IO8<D> + IO8<S>,
    {
        step!((), {
            0: if let Some(v) = self.read8(bus, src) {
              VAL8.store(v, Relaxed);
              go!(1);
            },
            1: if self.write8(bus, dst, VAL8.load(Relaxed)).is_some() {
              go!(2);
             },
            2: {
              go!(0);
              self.fetch(bus);
            },
        });
    }
    // sの値をdに格納する (16 bit)
    pub fn ld16<D: Copy, S: Copy>(&mut self, bus: &mut Peripherals, dst: D, src: S)
    where
        Self: IO16<D> + IO16<S>,
    {
        step!((), {
        0: if let Some(v) = self.read16(bus, src) {
          VAL16.store(v, Relaxed);
          go!(1);
        },
        1: if self.write16(bus, dst, VAL16.load(Relaxed)).is_some() {
          go!(2);
        },
        2: {
          go!(0);
          self.fetch(bus);
          },
        });
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
