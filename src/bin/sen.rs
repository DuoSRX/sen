extern crate sen;
// use std::error::Error;
// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;
use sen::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
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
