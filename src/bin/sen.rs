extern crate sen;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use sen::cpu::Cpu;

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/donkeykong.nes");

    let mut file = File::open(path).unwrap();

    let mut rom: Vec<u8> = Vec::new();
    file.read_to_end(&mut rom).unwrap();

    let mut cpu = Cpu::new();
    let mut i = 0;
    for byte in rom.iter().skip(16).take(20) {
        cpu.ram.val[i] = *byte;
        println!("{:x}", byte);
        i += 1;
    }

    cpu.step();
    cpu.step();
    cpu.step();
    cpu.step();
    cpu.step();
    cpu.step();
    cpu.step();
    cpu.step();
    // println!("{:?}", cpu);
    // println!("Flags: {:08b}", cpu.regs.p);
    // cpu.ram.val[0] = 0x4E;
    // cpu.ram.val[1] = 0x45;
    // cpu.ram.val[2] = 0x53;
    //
    // cpu.step();
    // println!("{:?}", cpu);
    // println!("Flags: {:08b}", cpu.regs.p);
    //
    // cpu.step();
    // println!("{:?}", cpu);
    // println!("Flags: {:08b}", cpu.regs.p);
}
