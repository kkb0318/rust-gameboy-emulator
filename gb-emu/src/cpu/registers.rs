#[derive(Clone, Copy, Debug, Default)]
pub struct Registers {
    /// プログラムカウンタ．CPU が次に実行する命令のアドレスを格納する
    pub pc: u16,
    /// スタックポインタ
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    // フラグレジスタ．一部の演算命令において特定の状態になるとこのレジスタの指定のbit が1 になる．下位4 bit は使われておらず常に0
    pub f: u8,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }
    pub fn write_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xF0) as u8; // 下位4bitは常に0
    }
    pub fn write_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8
    }
    pub fn write_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8
    }
    pub fn write_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8
    }
    // Fレジスタの各bit(Z, N, H, C)が示す状態
    // Z 演算結果が0 の場合に1 になる
    pub fn zf(&self) -> bool {
        (self.f & 0b_1000_0000) > 0
    }
    // N 減算命令の場合に1 になる
    pub fn nf(&self) -> bool {
        (self.f & 0b_0100_0000) > 0
    }
    // H 3 bit 目で繰り上（下）がりが発生すると1 になる
    pub fn hf(&self) -> bool {
        (self.f & 0b_0010_0000) > 0
    }
    // C 最上位bit（7 bit 目）で繰り上（下）がりが発生すると1 になる
    pub fn cf(&self) -> bool {
        (self.f & 0b_0001_0000) > 0
    }
    pub fn set_zf(&mut self, zf: bool) {
        if zf {
            self.f |= 0b_1000_0000;
        } else {
            self.f &= 0b_0111_1111;
        }
    }
    pub fn set_nf(&mut self, nf: bool) {
        if nf {
            self.f |= 0b_0100_0000;
        } else {
            self.f &= 0b_1011_1111;
        }
    }
    pub fn set_hf(&mut self, hf: bool) {
        if hf {
            self.f |= 0b_0010_0000;
        } else {
            self.f &= 0b_1101_1111;
        }
    }
    pub fn set_cf(&mut self, cf: bool) {
        if cf {
            self.f |= 0b_0001_0000;
        } else {
            self.f &= 0b_1110_1111;
        }
    }
}
