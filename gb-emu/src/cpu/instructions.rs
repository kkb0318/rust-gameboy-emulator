use super::operand::{IO16, IO8};
use super::Cpu;
use crate::peripherals::Peripherals;
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering::Relaxed};

impl Cpu {
    /// no operation  何もせず次の命令をfetchするだけ
    pub fn nop(&mut self, bus: &mut Peripherals) {
        self.fetch(bus)
    }

    /// LD命令
    /// メモリやレジスタ間でのデータ転送
    /// sの値をdに格納する (8 bit)
    /// where句により、SelfがD, Sに対するIO16を実装しているというトレイト境界が設定されている
    /// これにより、D, SはReg8, Imm8 といった、IO8が実装されている型のみ許容される
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
    /// メモリやレジスタ間でのデータ転送
    /// sの値をdに格納する (16 bit)
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
    /// CP命令
    /// A レジスタとs の値の比較
    /// （A レジスタからsの値を引き フラグレジスタを適切に設定. 演算結果は格納されない
    /// Z 演算結果が0 の場合は1 にする
    /// N 無条件に1 にする
    /// H 4 bit 目からの繰り下がりが発生した場合は1 にする
    /// C 8 bit 目からの繰り下がりが発生した場合は1 にする
    pub fn cp<S: Copy>(&mut self, bus: &Peripherals, src: S)
    where
        Self: IO8<S>,
    {
        if let Some(v) = self.read8(bus, src) {
            let (result, carry) = self.regs.a.overflowing_sub(v);
            self.regs.set_zf(result == 0);
            self.regs.set_nf(true);
            self.regs.set_hf((self.regs.a & 0xf) < (v & 0xf));
            self.regs.set_cf(carry);
            self.fetch(bus);
        }
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
