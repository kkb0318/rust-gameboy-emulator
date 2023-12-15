use super::operand::{Cond, Reg16, IO16, IO8};
use super::Cpu;
use crate::cpu::operand::{Imm16, Imm8};
use crate::peripherals::Peripherals;
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering::Relaxed};

impl Cpu {
    /// no operation  何もせず次の命令をfetchするだけ
    pub fn nop(&mut self, bus: &mut Peripherals) {
        self.fetch(bus)
    }
    fn sub_general(&mut self, val: u8, carry: bool) -> u8 {
        let cy = carry as u8;
        let result = self.regs.a.wrapping_sub(val).wrapping_sub(cy);
        self.regs.set_zf(result == 0);
        self.regs.set_nf(true);
        self.regs.set_hf((self.regs.a & 0xf) < (val & 0xf) + cy);
        self.regs
            .set_cf((self.regs.a as u16) < (val as u16) + (cy as u16));
        result
    }
    fn rlc_general(&mut self, val: u8) -> u8 {
        self.regs.set_zf(val == 0);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(val & 0x80 > 0);
        (val << 1) | (val >> 7)
    }
    fn rl_general(&mut self, val: u8) -> u8 {
        let new_val = (val << 1) | self.regs.cf() as u8;
        self.regs.set_zf(new_val == 0);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(val & 0x80 > 0);
        new_val
    }
    fn rrc_general(&mut self, val: u8) -> u8 {
        self.regs.set_zf(val == 0);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(val & 1 > 0);
        (val << 7) | (val >> 1)
    }
    fn rr_general(&mut self, val: u8) -> u8 {
        let new_val = ((self.regs.cf() as u8) << 7) | (val >> 1);
        self.regs.set_zf(new_val == 0);
        self.regs.set_nf(false);
        self.regs.set_hf(false);
        self.regs.set_cf(val & 1 > 0);
        new_val
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
            self.regs.set_hf((self.regs.a & 0xf) < (v & 0xf)); // self.regs.a と 0xf（16進数で15、2進数で1111）のビットごとのAND演算。これにより、self.regs.a の下位4ビットが取り出される
            self.regs.set_cf(carry);
            self.fetch(bus);
        }
    }
    /// s をインクリメント（s の値に1 足した値をs に格納）．
    pub fn inc<S: Copy>(&mut self, bus: &mut Peripherals, src: S)
    where
        Self: IO8<S>,
    {
        step!((), {
          0: if let Some(v) = self.read8(bus, src) {
            let new_val = v.wrapping_add(1);
            self.regs.set_zf(new_val == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(v & 0xf == 0xf);
            VAL8.store(new_val, Relaxed);
            go!(1);
          },
          1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn inc16<S: Copy>(&mut self, bus: &mut Peripherals, src: S)
    where
        Self: IO16<S>,
    {
        step!((), {
          0: if let Some(v) = self.read16(bus, src) {
            VAL16.store(v.wrapping_add(1), Relaxed);
            go!(1);
          },
          1: if self.write16(bus, src, VAL16.load(Relaxed)).is_some() {
            return go!(2);
          },
          2: {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    /// s をデクリメント（s の値に1 引いた値をs に格納）．
    pub fn dec<S: Copy>(&mut self, bus: &mut Peripherals, src: S)
    where
        Self: IO8<S>,
    {
        step!((), {
          0: if let Some(v) = self.read8(bus, src) {
            let new_val = v.wrapping_sub(1);
            self.regs.set_zf(new_val == 0);
            self.regs.set_nf(true);
            self.regs.set_hf(v & 0xf == 0);
            VAL8.store(new_val, Relaxed);
            go!(1);
          },
          1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn dec16<S: Copy>(&mut self, bus: &mut Peripherals, src: S)
    where
        Self: IO16<S>,
    {
        step!((), {
          0: if let Some(v) = self.read16(bus, src) {
            VAL16.store(v.wrapping_sub(1), Relaxed);
            go!(1);
          },
          1: if self.write16(bus, src, VAL16.load(Relaxed)).is_some() {
            return go!(2);
          },
          2: {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn rl<S: Copy>(&mut self, bus: &mut Peripherals, src: S)
    where
        Self: IO8<S>,
    {
        step!((), {
          0: if let Some(v) = self.read8(bus, src) {
            VAL8.store(self.rl_general(v), Relaxed);
            go!(1);
          },
          1: if self.write8(bus, src, VAL8.load(Relaxed)).is_some() {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn bit<S: Copy>(&mut self, bus: &Peripherals, bit: usize, src: S)
    where
        Self: IO8<S>,
    {
        if let Some(mut v) = self.read8(bus, src) {
            v &= 1 << bit;
            self.regs.set_zf(v == 0);
            self.regs.set_nf(false);
            self.regs.set_hf(true);
            self.fetch(bus);
        }
    }
    pub fn push(&mut self, bus: &mut Peripherals, src: Reg16) {
        step!((), {
          0: {
            VAL16.store(self.read16(bus, src).unwrap(), Relaxed);
            go!(1);
          },
          1: if self.push16(bus, VAL16.load(Relaxed)).is_some() {
            go!(2);
          },
          2: {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn push16(&mut self, bus: &mut Peripherals, val: u16) -> Option<()> {
        step!(None, {
          0: {
            go!(1);
            return None;
          },
          1: {
            let [lo, hi] = u16::to_le_bytes(val);
            self.regs.sp = self.regs.sp.wrapping_sub(1);
            bus.write(self.regs.sp, hi);
            VAL8.store(lo, Relaxed);
            go!(2);
            return None;
          },
          2: {
            self.regs.sp = self.regs.sp.wrapping_sub(1);
            bus.write(self.regs.sp, VAL8.load(Relaxed));
            go!(3);
            return None;
          },
          3: return Some(go!(0)),
        });
    }
    pub fn pop(&mut self, bus: &mut Peripherals, dst: Reg16) {
        if let Some(v) = self.pop16(bus) {
            self.write16(bus, dst, v);
            self.fetch(bus);
        }
    }
    pub fn pop16(&mut self, bus: &Peripherals) -> Option<u16> {
        step!(None, {
          0: {
            VAL8.store(bus.read(self.regs.sp), Relaxed);
            self.regs.sp = self.regs.sp.wrapping_add(1);
            go!(1);
            return None;
          },
          1: {
            let hi = bus.read(self.regs.sp);
            self.regs.sp = self.regs.sp.wrapping_add(1);
            VAL16.store(u16::from_le_bytes([VAL8.load(Relaxed), hi]), Relaxed);
            go!(2);
            return None;
          },
          2: {
            go!(0);
            return Some(VAL16.load(Relaxed));
          },
        });
    }
    pub fn jr(&mut self, bus: &Peripherals) {
        step!((), {
          0: if let Some(v) = self.read8(bus, Imm8) {
            self.regs.pc = self.regs.pc.wrapping_add(v as i8 as u16);
            return go!(1);
          },
          1: {
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn jr_c(&mut self, bus: &Peripherals, cond: Cond) {
        step!((), {
          0: if let Some(v) = self.read8(bus, Imm8) {
            go!(1);
            if self.cond(cond) {
              self.regs.pc = self.regs.pc.wrapping_add(v as i8 as u16);
              return;
            }
          },
          1: {
            go!(0);
            self.fetch(bus);
          },
        });
    }

    fn cond(&self, cond: Cond) -> bool {
        match cond {
            Cond::NZ => !self.regs.zf(),
            Cond::Z => self.regs.zf(),
            Cond::NC => !self.regs.cf(),
            Cond::C => self.regs.cf(),
        }
    }
    pub fn call(&mut self, bus: &mut Peripherals) {
        step!((), {
          0: if let Some(v) = self.read16(bus, Imm16) {
            VAL16.store(v, Relaxed);
            go!(1);
          },
          1: if self.push16(bus, self.regs.pc).is_some() {
            self.regs.pc = VAL16.load(Relaxed);
            go!(0);
            self.fetch(bus);
          },
        });
    }
    pub fn ret(&mut self, bus: &Peripherals) {
        step!((), {
          0: if let Some(v) = self.pop16(bus) {
            self.regs.pc = v;
            return go!(1);
          },
          1: {
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
