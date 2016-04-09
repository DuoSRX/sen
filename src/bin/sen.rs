extern crate sen;

use std::fs::File;
use std::path::Path;

use sen::cpu::Cpu;
use sen::ppu::Ppu;
use sen::cartridge::Cartridge;

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/roms/donkeykong.nes");
    //let path = Path::new("/Users/xavier/code/rust/sen/roms/galaxian.nes");

    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);

    println!("{}", cartridge.header);

    let ppu = Ppu::new();
    let mut cpu = Cpu::new(cartridge, ppu);
    cpu.reset();

    loop {
        cpu.step();
    }
}
