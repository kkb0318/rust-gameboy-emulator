pub struct WRam(Box<[u8; 8192]>);

impl WRam {
    pub fn new() -> Self {
        Self(Box::new([0; 8192]))
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.0[(addr as usize) % 8192]
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        self.0[(addr as usize) % 8192] = val;
    }
}
