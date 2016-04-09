// http://wiki.nesdev.com/w/index.php/PPU_programmer_reference

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
    address: u8, // aaaa aaaa 0x2006
    data: u8, // dddd dddd 0x2007
    oam_dma: u8 // aaaa aaaa 0x4014
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            control: 0,
            mask: 0,
            status: 0,//0x80,
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
    regs: Registers
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            regs: Registers::new()
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        //panic!("PPU::load({:04x}) not implemented yet", address);
        match address {
            0x2002 => self.regs.status,
            _ => { panic!("oops"); }
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
            0x2006 => { self.regs.address = value }
            0x2007 => { self.regs.data = value }
            _ => panic!("PPU::store({:04x} at {:04x}) not implemented yet", value, address)
        };
    }
}
