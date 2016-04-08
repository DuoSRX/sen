extern crate sen;

use std::fs::File;
use std::path::Path;

use sen::cpu::Cpu;
use sen::cartridge::Cartridge;

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/roms/donkeykong.nes");

    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);

    println!("{}", cartridge.header);

    let mut cpu = Cpu::new();
    cpu.step();

    // cpu.step();
    // println!("{:?}", cpu);
    // println!("Flags: {:08b}", cpu.regs.p);
}
