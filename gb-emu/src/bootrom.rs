pub struct Bootrom {
    rom: Box<[u8]>,
    active: bool,
}

impl Bootrom {
    pub fn new(rom: Box<[u8]>) -> Self {
        Self { rom, active: true }
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn write(&mut self, _: u16, val: u8) {
        // ビットごとのAND演算を行い、結果を左辺の変数に代入する
        // val=0のときはtrue(1)との演算のためかわらないが
        // val!=0のときはfalse(0)との演算のため常にactiveがfalse(0)になる
        self.active &= val == 0;
    }
}
