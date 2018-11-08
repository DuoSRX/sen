use std;

use cartridge::Cartridge;
use controller::Controller;
use ppu::Ppu;

trait Memory {
    fn load(address: u16) -> u8;
    fn store(address: u16, value: u8) -> u8;
}

pub struct CpuMemory {
    pub ram: Ram,
    pub cartridge: Cartridge,
    pub ppu: Ppu,
    pub controller: Controller
    // TODO: apu
}

impl CpuMemory {
    pub fn new(cartridge: Cartridge, ppu: Ppu, controller: Controller) -> CpuMemory {
        CpuMemory {
            cartridge: cartridge,
            ppu: ppu,
            controller: controller,
            ram: Ram::new()
        }
    }

    pub fn load(&mut self, address: u16) -> u8 {
        if address < 0x2000 {
            return self.ram.load(address);
        } else if address < 0x4000 {
            return self.ppu.load(0x2000 + address % 8);
        } else if address == 0x4016 {
            return self.controller.load(address);
        } else if address < 0x4018 {
            // TODO: APU
            return 0;
        } else if address < 0x6000 {
            println!("Reading from memory at {:04x} - Not implemented yet", address);
            return 0;
            //panic!("Address loading at {:04x} not implemented", address);
        } else if self.cartridge.header.prg_size > 1 {
            return self.cartridge.prg[address as usize & 0x7FFF];
        } else {
            return self.cartridge.prg[address as usize & 0x3FFF];
        };
    }

    pub fn store(&mut self, address: u16, value: u8) {
        if address < 0x2000 {
            self.ram.store(address, value);
        } else if address < 0x4000 {
            self.ppu.store(0x2000 + address % 8, value);
        } else if address == 0x4014 {
            self.dma(value as u16);
        } else if address == 0x4016 {
            self.controller.store(address, value);
        } else if address < 0x4018 {
            // TODO: APU
        } else if address < 0x6000 {
            println!("Writing {:02x} to memory at {:04x} - Not implemented yet", value, address);
            //panic!("Address storing at {:04x} not implemented", address);
        } else {
            // FIXME: Yeah. This should go to a mapper. This does not work correctly;
            // Can you even store in the PRG anyway...?
            self.cartridge.prg[address as usize & 0x3FFF] = value;
        };
    }

    fn dma(&mut self, start: u16) {
        let page = start * 0x100;

        for address in 0..256 {
            let value = self.ram.load(page + address);
            self.ppu.oam_data[address as usize] = value;
        }
    }
}

pub struct Ram {
    pub val: Vec<u8>,
}

impl std::fmt::Debug for Ram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.val[0..10]))
    }
}

impl Ram {
    pub fn new() -> Ram {
        Ram { val: vec![0; 0xFFFF] }
    }

    pub fn load(&self, address: u16) -> u8 {
        self.val[address as usize & 0x7ff]
    }

    pub fn store(&mut self, address: u16, value: u8) {
        self.val[address as usize & 0x7ff] = value;
    }
}
