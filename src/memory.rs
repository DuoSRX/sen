use std;

use cartridge::Cartridge;
use ppu::Ppu;

trait Memory {
    fn load(address: u16) -> u8;
    fn store(address: u16, value: u8) -> u8;
}

pub struct CpuMemory {
    pub ram: Ram,
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    // TODO: apu
    // TODO: controllers
}

impl CpuMemory {
    pub fn new(cartridge: Cartridge, ppu: Ppu) -> CpuMemory {
        CpuMemory {
            cartridge: cartridge,
            ppu: ppu,
            ram: Ram::new()
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        //println!("Address: {:02x} ({})", address, address);
        if address < 0x2000 {
            return self.ram.load(address);
        } else if address < 0x4000 {
            return self.ppu.load(0x2000 + address % 8);
        } else if address < 0x6000 {
            println!("Reading from PPU at {:04x}", address);
            return 0;
            //panic!("Address loading at {:04x} not implemented", address);
        } else {
            // FIXME: Yeah. This should go to a mapper. This does not work correctly;
            //println!("{:02x}", self.cartridge.prg[address as usize & 0x3FFF]);
            return self.cartridge.prg[address as usize & 0x3FFF];
        };
    }

    pub fn store(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.ram.store(address, value);
        } else if address < 0x4000 {
            self.ppu.store(0x2000 + address % 8, value);
        } else if address == 0x4014 {
            self.dma();
        } else if address < 0x6000 {
            println!("Writing {:08b} to PPU at {:04x} (Not implemented yet)", value, address);
            //panic!("Address storing at {:04x} not implemented", address);
        } else {
            // TODO: Move to a mapper module?
            // FIXME: Yeah. This should go to a mapper. This does not work correctly;
            // Can you even store in the PRG anyway...?
            // What about the CHR?
            self.cartridge.prg[address as usize & 0x7FFF] = value;
        };
    }

    fn dma(&mut self) {
        let a = 0x4014 << 8;

        for address in a..a + 256 {
            let value = self.load(address);
            self.store(0x2004, value);
        }
    }
}

pub struct Ram {
    pub val: [u8; 0xFFFF]
}

impl std::fmt::Debug for Ram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Ram {
    pub fn new() -> Ram {
        Ram { val: [0; 0xFFFF] }
    }

    pub fn load(&self, address: u16) -> u8 {
        self.val[address as usize & 0x7ff]
    }

    pub fn store(&mut self, address: u16, value: u8) {
        self.val[address as usize & 0x7ff] = value;
    }
}
