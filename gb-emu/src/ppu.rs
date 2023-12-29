use std::iter;

use crate::{LCD_PIXELS, LCD_WIDTH};

#[derive(Copy, Clone, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

const BG_WINDOW_ENABLE: u8 = 1 << 0;
const SPRITE_ENABLE: u8 = 1 << 1;
const SPRITE_SIZE: u8 = 1 << 2;
const BG_TILE_MAP: u8 = 1 << 3;
const TILE_DATA_ADDRESSING_MODE: u8 = 1 << 4;
const WINDOW_ENABLE: u8 = 1 << 5;
const WINDOW_TILE_MAP: u8 = 1 << 6;
const PPU_ENABLE: u8 = 1 << 7;

const LYC_EQ_LY: u8 = 1 << 2;
const HBLANK_INT: u8 = 1 << 3;
const VBLANK_INT: u8 = 1 << 4;
const OAM_SCAN_INT: u8 = 1 << 5;
const LYC_EQ_LY_INT: u8 = 1 << 6;

const BANK: u8 = 1 << 3;
const PALETTE: u8 = 1 << 4;
const X_FLIP: u8 = 1 << 5;
const Y_FLIP: u8 = 1 << 6;
const OBJ2BG_PRIORITY: u8 = 1 << 7;

pub struct Ppu {
    mode: Mode,
    lcdc: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,
    wly: u8,
    vram: Box<[u8; 0x2000]>,
    bcps: u8,
    ocps: u8,
    vbk: u8,
    vram2: Box<[u8; 0x2000]>,
    oam: Box<[u8; 0xA0]>,
    pub oam_dma: Option<u16>,
    pub hdma_src: u16,
    hdma_dst: u16,
    pub hblank_dma: Option<u16>,
    pub general_dma: Option<u16>,
    bg_palette_memory: Box<[u8; 0x40]>,
    sprite_palette_memory: Box<[u8; 0x40]>,
    cycles: u8,
    buffer: Box<[u8; LCD_PIXELS * 4]>,
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: Mode::OamScan,
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0x00,
            obp0: 0x00,
            obp1: 0x00,
            wy: 0,
            wx: 0,
            wly: 0,
            vram: Box::new([0; 0x2000]),
            bcps: 0,
            ocps: 0,
            vbk: 0,
            vram2: Box::new([0; 0x2000]),
            oam: Box::new([0; 0xA0]),
            oam_dma: None,
            hdma_src: 0,
            hdma_dst: 0,
            hblank_dma: None,
            general_dma: None,
            bg_palette_memory: Box::new([
                0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29,
                0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56,
                0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F,
                0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00,
                0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00,
            ]),
            sprite_palette_memory: Box::new([
                0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29,
                0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56,
                0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F,
                0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00, 0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00,
                0xFF, 0x7F, 0xB5, 0x56, 0x4A, 0x29, 0x00, 0x00,
            ]),
            cycles: 20,
            buffer: Box::new([0; LCD_PIXELS * 4]),
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => {
                if self.mode == Mode::Drawing {
                    0xFF // モード3の間はVRAMにアクセスできない
                } else {
                    self.vram[addr as usize & 0x1FFF]
                }
            }
            0xFE00..=0xFE9F => {
                if self.mode == Mode::Drawing || self.mode == Mode::OamScan {
                    0xFF // モード2 ，モード3の間はOAMにアクセスできない
                } else {
                    self.oam[addr as usize & 0xFF]
                }
            }
            0xFF40 => self.lcdc,
            0xFF41 => 0x80 | self.stat | self.mode as u8, // 7bit目は常に1
            // 他のレジスタも同じように実装
            _ => unreachable!(),
        }
    }
    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9FFF => {
                if self.mode != Mode::Drawing {
                    self.vram[addr as usize & 0x1FFF] = val;
                }
            }
            0xFE00..=0xFE9F => {
                if self.mode != Mode::Drawing && self.mode != Mode::OamScan {
                    self.oam[addr as usize & 0xFF] = val;
                }
            }
            0xFF40 => self.lcdc = val,
            0xFF41 => self.stat = (self.stat & LYC_EQ_LY) | (val & 0xF8), // 0～2bit目は書き込み不可
            0xFF44 => {} // LYレジスタは書き込み不可
            // 他のレジスタも同じように実装
            _ => unreachable!(),
        }
    }
    fn get_pixel_from_tile(&self, tile_idx: usize, row: u8, col: u8) -> u8 {
        let r = (row * 2) as usize; // タイルは1行(8ピクセル)あたり16bit(2B)
        let c = (7 - col) as usize; // col列目は(7 - col)bit目
        let tile_addr = tile_idx << 4; // タイルの開始アドレスはタイルのインデックスの16倍
        let low = self.vram[(tile_addr | r) & 0x1FFF]; // ピクセルの上位bit(8ピクセル分)
        let high = self.vram[(tile_addr | (r + 1)) & 0x1FFF]; // 下位bit(8ピクセル分)
        (((high >> c) & 1) << 1) | ((low >> c) & 1) // ピクセルの値
    }
    fn get_tile_idx_from_tile_map(&self, tile_map: bool, row: u8, col: u8) -> usize {
        let start_addr: usize = 0x1800 | ((tile_map as usize) << 10);
        let ret = self.vram[start_addr | (((row as usize) << 5) + col as usize) & 0x3FF];
        if self.lcdc & TILE_DATA_ADDRESSING_MODE > 0 {
            ret as usize
        } else {
            ((ret as i8 as i16) + 0x100) as usize
        }
    }
    fn render_bg(&mut self) {
        if self.lcdc & BG_WINDOW_ENABLE == 0 {
            return;
        }
        let y = self.ly.wrapping_add(self.scy); // 表示領域が256を超えた場合は回り込む
        for i in 0..LCD_WIDTH {
            let x = (i as u8).wrapping_add(self.scx); // 表示領域が256を超えた場合は回り込む

            let tile_idx = self.get_tile_idx_from_tile_map(
                self.lcdc & BG_TILE_MAP > 0, // どちらのタイルマップを使うか
                y >> 3,
                x >> 3, // タイルのサイズは8×8
            );

            let pixel = self.get_pixel_from_tile(tile_idx, y & 7, x & 7);

            self.buffer[LCD_WIDTH * self.ly as usize + i] = match (self.bgp >> (pixel << 1)) & 0b11
            {
                // パレットから色を取得
                0b00 => 0xFF, // 白
                0b01 => 0xAA, // ライトグレー
                0b10 => 0x55, // ダークグレー
                _ => 0x00,    // 黒
            };
        }
    }
    fn check_lyc_eq_ly(&mut self) {
        if self.ly == self.lyc {
            self.stat |= LYC_EQ_LY;
        } else {
            self.stat &= !LYC_EQ_LY;
        }
    }
    pub fn emulate_cycle(&mut self) -> bool {
        if self.lcdc & PPU_ENABLE == 0 {
            // PPUが無効化されている場合は何もしない
            return false;
        }

        self.cycles -= 1; // cycleの値を更新する
        if self.cycles > 0 {
            // 最終cycleでない場合は何もしない
            return false;
        }

        let mut ret = false; // VSYNCであるかを示す変数
        match self.mode {
            Mode::HBlank => {
                self.ly += 1; // HBlankの終わりは行の終わりなのでLYをインクリメント
                if self.ly < 144 {
                    // 描画する行が残っている場合は次のモードはOAM Scan
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                } else {
                    // その行がVBlankの手前の行だった場合は次のモードはVBlank
                    self.mode = Mode::VBlank;
                    self.cycles = 114;
                }
                self.check_lyc_eq_ly(); // LYを更新したら必ずLYCと等しいかを確認する
            }
            Mode::VBlank => {
                self.ly += 1; // VBlankの終わりは行の終わりなのでLYをインクリメント
                if self.ly > 153 {
                    // VBlankの最後の行だった場合は次のモードはOAM Scan
                    ret = true; // VBlankの最後はVSYNCのタイミング
                    self.ly = 0; // 先頭の行に戻る
                    self.mode = Mode::OamScan;
                    self.cycles = 20;
                } else {
                    // VBlankの最後の行ではなかった場合はまだVBlank
                    self.cycles = 114;
                }
                self.check_lyc_eq_ly(); // LYを更新したら必ずLYCと等しいかを確認する
            }
            Mode::OamScan => {
                // 次のモードはDrawing Pixels
                self.mode = Mode::Drawing;
                self.cycles = 43;
            }
            Mode::Drawing => {
                // 次のモードはHBlank
                self.render_bg(); // Drawing Pixelsの最終cycleなのでレンダリングを実行
                self.mode = Mode::HBlank;
                self.cycles = 51;
            }
        }
        ret
    }
    pub fn pixel_buffer(&self) -> Box<[u8]> {
        self.buffer
            .iter()
            .flat_map(|&e| iter::repeat(e.into()).take(3))
            .collect::<Box<[u8]>>()
    }
}
