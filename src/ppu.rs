use std;

use cartridge::Cartridge;

// http://wiki.nesdev.com/w/index.php/PPU_programmer_reference

#[allow(dead_code)]
mod ppumask {
    pub const BACKGROUND:      u8 = 0b0001000;
    pub const SPRITES:         u8 = 0b0010000;
}

pub struct Vram {
    pub val: [u8; 0x2000]//[u8; 0x800]
}

impl std::fmt::Debug for Vram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Vram {
    pub fn load(&self, address: u16) -> u8 {
        //self.cartridge.chr[address]
        unimplemented!()
    }

    pub fn store(&mut self, address: u16, value: u8) {
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
    data: u8, // dddd dddd 0x2007
    oam_dma: u8 // aaaa aaaa 0x4014
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
            data: 0,
            oam_dma: 0
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

    cycle: u64,
    scanline: u16, // 0-239 is visible, 240 post, 241-260 vblank, 261 pre
    frame: u64,

    palette: [u8; 32],
    name_table: [u8; 2048],
    oam_data: [u8; 256]
}

impl Ppu {
    pub fn new(cartridge: Cartridge) -> Ppu {
        Ppu {
            cartridge: cartridge,
            regs: Registers::new(),
            vram: Vram { val: [0; 0x2000] },
            vram_rw_high: true,
            scroll_x: 0,
            scroll_y: 0,

            cycle: 340,
            scanline: 240,
            frame: 0,

            palette: [0; 32],
            name_table: [0; 2048],
            oam_data: [0; 256]
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline = 240;
        self.frame = 0;
        self.regs.control = 0;
        self.regs.mask = 0;
        self.regs.oam_address = 0;
    }

    // #[allow(dead_code)]
    // fn tick(&mut self) {
    //     // trigger NMI in specific cases
    //     // update cycle/scanline/frame
    // }
    //
    // #[allow(dead_code)]
    // fn step(&mut self) {
    //     self.tick();
    //     // TODO: Implement the rest
    //
    //     let rendering = self.regs.mas
    //
    //     loop {
    //         let scanline = self.cycle + 114;
    //
    //         if self.scanline < 240 {
    //             // render scanline
    //         }
    //
    //         self.scanline += 1;
    //
    //         if self.scanline == 241 {
    //             // vblank
    //         } else if self.scanline == 261 {
    //             // new frame
    //             // set vblank bit in PPUSTATUS
    //             self.scanline = 0;
    //         }
    //
    //         self.cycle += 114;
    //     }
    // }

    pub fn vram_load(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            self.cartridge.chr[address as usize]
        } else {
            panic!("Reading VRam at {:04x} is not implemented yet!");
        }
    }

    pub fn vram_store(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.cartridge.chr[address as usize] = value;
        } else {
            panic!("Storing VRam at {:04x} is not implemented yet!");
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        //panic!("PPU::load({:04x}) not implemented yet", address);
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

    fn address_increment(&mut self) -> u16 {
        if self.regs.status & 0x04 == 0 {
            1
        } else {
            32
        }
    }

    // $2002 Read from PPUSTATUS
    fn read_status(&mut self) -> u8 {
        let status = self.regs.status;
        self.regs.status ^= 0x80; // Clear VBlank bit
        status
    }

    // $2007 Read from PPUDATA
    fn read_data(&mut self) -> u8 {
        // TODO: Handle buffered reads
        let address = self.regs.address;
        let value = self.vram_load(address);
        self.regs.address += self.address_increment();
        value
    }

    // $2007 Write to PPUDATA
    fn write_data(&mut self, value: u8) {
        let address = self.regs.address as u16;
        self.vram_store(address, value);
        self.regs.address += self.address_increment();
    }

    // $2004 Read from OAMDATA
    fn read_oam_data(&mut self) -> u8 {
        let address = self.regs.oam_address as u16;
        self.vram_load(address)
    }

    // $2004 Write to OAMDATA
    fn write_oam_data(&mut self, value: u8) {
        let address = self.regs.oam_address as u16;
        self.vram_store(address, value);
        self.regs.oam_address += 1;
    }

    // $2006 Write to PPUADDR
    fn write_address(&mut self, address: u8) {
        if self.vram_rw_high {
            self.regs.address = (address as u16) << 8;
            self.vram_rw_high = false;
        } else {
            self.regs.address += address as u16;
            self.vram_rw_high = true;
        }
    }

    // $2005 Write to PPUSCROLL
    fn write_scroll(&mut self, value: u8) {
        if self.vram_rw_high {
            self.scroll_x = value;
            self.vram_rw_high = false;
        } else {
            self.scroll_y = value;
            self.vram_rw_high = true;
        }
    }
}
