use std;

use cartridge::Cartridge;

// http://wiki.nesdev.com/w/index.php/PPU_programmer_reference

pub struct PpuResult {
    pub new_frame: bool,
    pub nmi: bool
}

// Palette inspired by fogleman/nes
const PALETTE_RGB: [u32; 64] = [
    0x666666, 0x002A88, 0x1412A7, 0x3B00A4, 0x5C007E,
    0x6E0040, 0x6C0600, 0x561D00, 0x333500, 0x0B4800,
    0x005200, 0x004F08, 0x00404D, 0x000000, 0x000000,
    0x000000, 0xADADAD, 0x155FD9, 0x4240FF, 0x7527FE,
    0xA01ACC, 0xB71E7B, 0xB53120, 0x994E00, 0x6B6D00,
    0x388700, 0x0C9300, 0x008F32, 0x007C8D, 0x000000,
    0x000000, 0x000000, 0xFFFEFF, 0x64B0FF, 0x9290FF,
    0xC676FF, 0xF36AFF, 0xFE6ECC, 0xFE8170, 0xEA9E22,
    0xBCBE00, 0x88D800, 0x5CE430, 0x45E082, 0x48CDDE,
    0x4F4F4F, 0x000000, 0x000000, 0xFFFEFF, 0xC0DFFF,
    0xD3D2FF, 0xE8C8FF, 0xFBC2FF, 0xFEC4EA, 0xFECCC5,
    0xF7D8A5, 0xE4E594, 0xCFEF96, 0xBDF4AB, 0xB3F3CC,
    0xB5EBF2, 0xB8B8B8, 0x000000, 0x000000,
];

struct Sprite {
    pub x: u8,
    pub y: u8,
    pub index: u8,
    pub attributes: u8 // vhp---PP
}

enum Tiles {
    Tiles8(u16),
    Tiles16(u16, u16)
}

// // http://wiki.nesdev.com/w/index.php/PPU_OAM
impl Sprite {
    pub fn palette(&self) -> u8 {
        (self.attributes & 0x3) + 4
    }
    fn horizontal_flip(&self) -> bool { (self.attributes & 0x40) != 0 }
    fn vertical_flip(&self) -> bool { (self.attributes & 0x80) != 0 }

    pub fn get_tiles(&self, ppu: &Ppu) -> Tiles {
        if ppu.sprite_size() == 8 {
            Tiles::Tiles8(self.index as u16 | ppu.sprite_pattern_table_address())
        } else {
            // Ignore PPUCTRL and take bit 0 instead
            let mut address: u16 = self.index as u16 & !1;
            if (self.index & 1) != 0 {
                address += 0x1000;
            }
            Tiles::Tiles16(address as u16, address as u16 + 1)
        }
    }
}

pub struct Vram {
    pub val: Vec<u8>,
}

impl std::fmt::Debug for Vram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

// TODO: Replace the vram_load() calls with this
impl Vram {
    pub fn load(&self, _address: u16) -> u8 {
        unimplemented!()
    }

    pub fn store(&mut self, _address: u16, _value: u8) {
        unimplemented!()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
#[derive(Default)]
struct Registers {
    control: u8, // VPHB SINN 0x2000
    mask: u8, // BGRs bMmG 0x2001
    status: u8, // VSO- ---- 0x2002
    oam_address: u8, // aaaa aaaa 0x2003
    oam_data: u8, // dddd dddd 0x2004
    scroll: u8, // xxxx xxxx 0x2005
    address: u16, // aaaa aaaa 0x2006
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            control: 0,
            mask: 0,
            status: 0x80,
            oam_address: 0,
            oam_data: 0,
            scroll: 0,
            address: 0,
        }
    }

}

//#[derive(Debug)]
#[allow(dead_code)]
pub struct Ppu {
    cartridge: Cartridge,
    regs: Registers,
    vram: Vram,

    // whether to write to the high or low byte
    vram_rw_high: bool,
    scroll_x: u8,
    scroll_y: u8,
    next_scroll_x: bool,
    data_buffer: u8,

    pub cycle: u64,
    pub new_frame: bool,
    pub frame_content: Vec<u8>, //[u8; 256 * 240 * 3],
    scanline: u16, // 0-239 is visible, 240 post, 241-260 vblank, 261 pre
    pub frames: u64,

    palettes: [u8; 32],
    name_tables: Vec<u8>,
    pub oam_data: [u8; 256]
}

impl Ppu {
    pub fn new(cartridge: Cartridge) -> Ppu {
        Ppu {
            cartridge: cartridge,
            regs: Registers::new(),
            vram: Vram { val: vec![0; 0x800] },
            vram_rw_high: true,
            scroll_x: 0,
            scroll_y: 0,
            next_scroll_x: true,
            data_buffer: 0,

            cycle: 340,
            new_frame: false,
            frame_content: vec![0; 256 * 240 * 3],
            scanline: 240,
            frames: 0,

            palettes: [0; 32],
            name_tables: vec![0; 2048],
            oam_data: [0; 256]
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.regs.control = 0;
        self.regs.mask = 0;
        self.regs.oam_address = 0;
    }

    pub fn vram_load(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            self.cartridge.chr[address as usize]
        } else if address < 0x3F00 {
            self.name_tables[address as usize & 0x07FF]
        } else if address < 0x4000 {
            self.palettes[address as usize & 0x1F]
        } else {
            panic!("Reading VRam at 0x{:04x} is not valid!");
        }
    }

    pub fn vram_store(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            // Can you even write to CHR...?
        } else if address < 0x3F00 {
            self.name_tables[address as usize & 0x07FF] = value;
        } else if address < 0x4000 {
            self.palettes[address as usize & 0x1F] = value;
        } else {
            panic!("Storing value 0x{:02x} in VRam at 0x{:04x} is not valid!", value, address);
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        match address {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam_data(),
            0x2007 => self.read_data(),
            _ => { panic!("Can't read PPU at {:04x}", address); }
        }
    }

    pub fn store(&mut self, address: u16, value: u8) {
        match address {
            // TODO: Handle NMIs when writing flags and all
            0x2000 => { self.regs.control = value }
            0x2001 => { self.regs.mask = value }
            0x2003 => { self.regs.oam_address = value }
            0x2004 => self.write_oam_data(value),
            0x2005 => self.write_scroll(value),
            0x2006 => self.write_address(value),
            0x2007 => self.write_data(value),
            _ => panic!("PPU::store({:04x} at {:04x}) not implemented yet", value, address)
        };
    }

    // Registers
    fn address_increment(&mut self) -> u16 {
        match self.regs.control & 0x04 {
            0 => 1,
            _ => 32
        }
    }

    // $2002 Read from PPUSTATUS
    fn read_status(&mut self) -> u8 {
        let status = self.regs.status;
        self.regs.status ^= 0x80; // Clear VBlank bit
        status
    }

    // $2004 Read from OAMDATA
    fn read_oam_data(&mut self) -> u8 {
        let address = self.regs.oam_address as u16;
        self.oam_data[address as usize]
    }

    // $2004 Write to OAMDATA
    fn write_oam_data(&mut self, value: u8) {
        let address = self.regs.oam_address as u16;
        self.oam_data[address as usize] = value;
        self.regs.oam_address.wrapping_add(1);
    }

    // $2005 Write to PPUSCROLL
    fn write_scroll(&mut self, value: u8) {
        if self.next_scroll_x {
            self.scroll_x = value;
            self.next_scroll_x = false;
        } else {
            self.scroll_y = value;
            self.next_scroll_x = true;
        }
    }

    // $2006 Write to PPUADDR
    fn write_address(&mut self, address: u8) {
        if self.vram_rw_high {
            self.regs.address = (self.regs.address & 0xFF) | ((address as u16) << 8);
            self.vram_rw_high = false;
        } else {
            self.regs.address = (self.regs.address & 0xFF00) | (address as u16);
            self.vram_rw_high = true;
        }
    }

    // $2007 Read from PPUDATA
    fn read_data(&mut self) -> u8 {
        let address = self.regs.address;
        let value = self.vram_load(address);
        self.regs.address += self.address_increment();

        // http://wiki.nesdev.com/w/index.php/PPU_registers#Data_.28.242007.29_.3C.3E_read.2Fwrite
        if address < 0x3F00 {
            let buff = self.data_buffer;
            self.data_buffer = value;
            return buff;
        } else {
            return value
        }
    }

    // $2007 Write to PPUDATA
    fn write_data(&mut self, value: u8) {
        let address = self.regs.address as u16;
        self.vram_store(address & 0x3FFF, value);
        self.regs.address += self.address_increment();
    }

    // Rendering
    fn background_pattern_table_address(&self) -> u16 {
        match self.regs.control & 0x10 {
            0 => 0,
            _ => 0x1000,
        }
    }

    fn sprite_pattern_table_address(&self) -> u16 {
        match self.regs.control & 0x8 {
            0 => 0,
            _ => 0x1000,
        }
    }

    fn show_sprites(&self) -> bool { self.regs.mask & 0x08 != 0 }
    fn show_background(&self) -> bool { self.regs.mask & 0x10 != 0 }

    fn sprite_size(&self) -> u8 {
        match self.regs.control & 0x20 {
            0 => 8,
            _ => 16
        }
    }

    fn get_pixel(&mut self, x: u8, offset: u16) -> u8 {
        let p0 = self.vram_load(offset);
        let p1 = self.vram_load(offset + 8);
        let bit0 = (p0 >> (7 - ((x % 8) as u8))) & 1;
        let bit1 = (p1 >> (7 - ((x % 8) as u8))) & 1;
        (bit1 << 1) | bit0
    }

    fn get_background_pixel(&mut self, x: u8) -> u32 {
        let x_offset = x as u16 / 8;
        let y_offset = self.scanline as u16 / 8;
        let x2 = x % 8;
        let y2 = (self.scanline % 8) as u8;

        let tile_address = 0x2000 + 32 * y_offset + x_offset;
        let tile = self.vram_load(tile_address) as u16;

        let mut offset = (tile << 4) + y2 as u16;
        offset += self.background_pattern_table_address();
        let pixel = self.get_pixel(x2, offset);

        let block = y_offset / 4 * 8 + x_offset / 4;
        let attributes = self.vram_load(0x23C0 + block);
        let left = x_offset % 4 < 2;
        let top = y_offset % 4 < 2;

        let mut attribute_color = attributes;
        if !left && top {
            attribute_color = attributes >> 2;
        } else if left && !top {
            attribute_color = attributes >> 4;
        } else if !left && !top {
            attribute_color = attributes >> 6;
        }

        attribute_color &= 0x3;

        let color = (attribute_color << 2) | pixel;
        let palette_address = 0x3F00 + color as u16;
        let palette = self.vram_load(palette_address) & 0x3F;
        PALETTE_RGB[palette as usize]
    }

    fn get_sprite_pixel(&mut self, x: u8, _background: bool) -> Option<u32> {
        let mut visible_count = 0;

        for n in 0..64 {
            if visible_count >= 8 { break; } // Not sure if that's correct...

            let sprite = Sprite {
                y: self.oam_data[n * 4],
                index: self.oam_data[n * 4 + 1],
                attributes: self.oam_data[n * 4 + 2],
                x: self.oam_data[n * 4 + 3],
            };

            // FIXME: This doesn't really work for 8x16 sprites
            let size = self.sprite_size();
            // let on_scanline = (sprite.y as u16) < self.scanline && self.scanline < sprite.y as u16 + size as u16;
            let in_box = x >= sprite.x && (x as u16) < sprite.x as u16 + size as u16;
            let on_scanline = !(self.scanline < sprite.y as u16) && (self.scanline < sprite.y as u16 + 8);

            if in_box && on_scanline {
                visible_count += 1;

                let pixel;
                match sprite.get_tiles(self) {
                    Tiles::Tiles8(tile) | Tiles::Tiles16(tile, _) => {
                        let mut sprite_x = x - sprite.x;
                        let mut sprite_y = self.scanline as u8 - sprite.y;

                        if sprite.horizontal_flip() { sprite_x = 7 - sprite_x; }
                        if sprite.vertical_flip() { sprite_y = 7 - sprite_y; }

                        let mut offset = (tile << 4) + sprite_y as u16;
                        offset += self.sprite_pattern_table_address();
                        pixel = self.get_pixel(sprite_x, offset);
                    }
                    // _ => {}
                    // Tiles::Tiles16(_top, _bottom) => { }
                }

                if pixel == 0 { continue }; // transparent, let's not do anything

                // if visible_count == 0 && background {
                //     self.regs.status |= 0x40; // set sprite 0 hit
                // }
                // visible_count += 1;

                let color = (sprite.palette() << 2) | pixel;
                let palette_address = 0x3F00 + color as u16;
                let palette = self.vram_load(palette_address) & 0x3F;
                return Some(PALETTE_RGB[palette as usize]);
            }
        }

        None
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        self.frame_content[((y * 256 + x) * 3 + 2) as usize] = (color >> 16) as u8;
        self.frame_content[((y * 256 + x) * 3 + 1) as usize] = (color >> 8) as u8;
        self.frame_content[((y * 256 + x) * 3 + 0) as usize] = color as u8;
    }

    fn make_scanline(&mut self) {
        let scanline = self.scanline;

        for x in 0..255 {
            let mut background = false;
            if self.show_background() {
                let c = self.get_background_pixel(x);
                background = true;
                self.set_pixel(x as u32, scanline as u32, c);
            } else {
                self.set_pixel(x as u32, scanline as u32, 0);
            }

            if self.show_sprites() {
                match self.get_sprite_pixel(x, background) {
                    Some(color) => self.set_pixel(x as u32, scanline as u32, color),
                    _ => ()
                }
            }
        }
    }

    pub fn step(&mut self, cpu_cycle: u64) -> PpuResult {
        let mut result = PpuResult { new_frame: false, nmi: false };

        loop {
            let next_scanline = 114 + self.cycle;
            if next_scanline > cpu_cycle {
                break;
            }
            if self.scanline < 240 {
                self.make_scanline();
            }
            self.scanline += 1;

            if self.scanline == 241 { // VBlank
                self.regs.status |= 0x80;
                self.regs.control &= !0x40; // sprite zero hit
                if (self.regs.control | 0x80) != 0 {
                    result.nmi = true;
                }
            } else if self.scanline == 261 {
                result.new_frame = true;
                self.frames += 1;
                self.scanline = 0;
                self.regs.status &= !0x80;
            }

            self.cycle += 114;
        }

        self.cycle = 0;
        result
    }
}
