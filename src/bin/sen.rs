extern crate sen;

use std::fs::File;
use std::path::Path;

use sen::cpu::Cpu;
use sen::ppu::Ppu;
use sen::cartridge::Cartridge;
use sen::memory::CpuMemory;

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/roms/donkeykong.nes");
    //let path = Path::new("/Users/xavier/code/rust/sen/roms/galaxian.nes");
    //let path = Path::new("/Users/xavier/code/rust/sen/roms/nestest.nes");
    //let path = Path::new("/Users/xavier/code/rust/sen/roms/instr_test-v4/rom_singles/01-basics.nes");

    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);
    let mut file2 = File::open(path).unwrap();
    let cartridge2 = Cartridge::load(&mut file2);

    println!("{}", cartridge.header);

    let ppu = Ppu::new(cartridge2);
    let memory = CpuMemory::new(cartridge, ppu);
    let mut cpu = Cpu::new(memory);

    cpu.reset();

    let mut i = 0;
    loop {
        cpu.step();
        i += 1;
        if i > 100 {
            break;
        }
    }
}
