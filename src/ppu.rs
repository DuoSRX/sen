use std;

// http://wiki.nesdev.com/w/index.php/PPU_programmer_reference

pub struct Vram {
    pub val: [u8; 0xFFFF]//[u8; 0x800]
}

impl std::fmt::Debug for Vram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Vram {
    //FIXME: This is actually completely wrong. This should handle CHR, palettes, nametables...
    pub fn load(&self, address: u16) -> u8 {
        self.val[address as usize & 0x7ff]
    }

    //FIXME: This is actually completely wrong. This should handle CHR, palettes, nametables...
    pub fn store(&mut self, address: u16, value: u8) {
        self.val[address as usize & 0x7ff] = value;
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

#[derive(Debug)]
pub struct Ppu {
    regs: Registers,
    vram: Vram,

    // whether to write to the high or low byte
    vram_rw_high: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            regs: Registers::new(),
            vram: Vram { val: [0; 0xFFFF] },
            vram_rw_high: true
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        //panic!("PPU::load({:04x}) not implemented yet", address);
        match address {
            0x2002 => self.regs.status,
            0x2007 => self.read_data(),
            _ => { panic!("Can't read PPU at {:04x}", address); }
        }
    }

    pub fn store(&mut self, address: u16, value: u8) {
        //panic!("PPU::store({:04x} at {:04x}) not implemented yet", value, address);
        match address {
            0x2000 => { self.regs.control = value }
            0x2001 => { self.regs.mask = value }
            0x2003 => { self.regs.oam_address = value }
            0x2004 => { self.regs.oam_data = value }
            0x2005 => { self.regs.scroll = value }
            0x2006 => { self.write_address(value) }
            0x2007 => { self.regs.data = value }
            _ => panic!("PPU::store({:04x} at {:04x}) not implemented yet", value, address)
        };
    }

    fn read_data(&mut self) -> u8 {
        let value = self.vram.load(self.regs.address);
        let increment = if self.regs.status & 0x04 == 0 { 1 } else { 32 };
        self.regs.address += increment;
        value
    }

    fn write_address(&mut self, address: u8) {
        if self.vram_rw_high {
            self.regs.address = (address as u16) << 8;
            self.vram_rw_high = false;
        } else {
            self.regs.address += address as u16;
            self.vram_rw_high = true;
        }
    }
}
