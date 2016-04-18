extern crate sen;
extern crate sdl2;

use std::fs::File;
use std::path::Path;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;

use sen::cpu::Cpu;
use sen::ppu::Ppu;
use sen::cartridge::Cartridge;
use sen::memory::CpuMemory;

fn main() {
    let path = Path::new("/Users/xavier/code/rust/sen/roms/donkeykong.nes");
    // let path = Path::new("/Users/xavier/code/rust/sen/roms/galaxian.nes");
    // let path = Path::new("/Users/xavier/code/rust/sen/roms/nestest.nes");
    // let path = Path::new("/Users/xavier/code/rust/sen/roms/instr_test-v4/rom_singles/01-basics.nes");

    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);
    let mut file2 = File::open(path).unwrap();
    let cartridge2 = Cartridge::load(&mut file2);

    println!("{}", cartridge.header);

    let mut ppu = Ppu::new(cartridge2);
    // let memory = CpuMemory::new(cartridge, ppu);
    // let mut cpu = Cpu::new(memory);
    //
    // //ppu.reset();
    // cpu.reset();
    //
    // let mut i = 0;
    // loop {
    //     cpu.step();
    //     i += 1;
    //     if i > 2 {
    //         break;
    //     }
    // }

    // for i in 0..256 {
    //     if i % 8 == 0 {
    //         println!("");
    //         print!("{:04x}: ", i);
    //     }
    //     print!("{:02x} ", ppu.vram_load(i));
    // }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("wat", 1024, 768)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(255,255, 255));
    renderer.clear();

    const PIXEL_SIZE: u32 = 4;
    const RECT_SIZE: u32 = 32;
    const GRID_SIZE: i32 = 24;

    for n in 128..256 {
        let x_offset: i32 = ((n % GRID_SIZE) * RECT_SIZE as i32) + PIXEL_SIZE as i32;//n * RECT_SIZE as i32 + 4;
        let y_offset: i32 = (n / GRID_SIZE * RECT_SIZE as i32) + PIXEL_SIZE as i32;

        for y in 0..8 {
            //println!("{:08b} ", ppu.vram_load(i));
            let plane0 = ppu.vram_load(y as u16 + (n as u16 * 16));
            let plane1 = ppu.vram_load(y as u16 + (n as u16 * 16) + 8);

            for x in 0..8 {
                let bit0 = (plane0 >> ((7 - ((x % 8) as u8)) as usize)) & 1;
                let bit1 = (plane1 >> ((7 - ((x % 8) as u8)) as usize)) & 1;
                let result = (bit1 << 1) | bit0;

                match result {
                    1 => renderer.set_draw_color(Color::RGB(215,215,215)),
                    2 => renderer.set_draw_color(Color::RGB(190,190,190)),
                    3 => renderer.set_draw_color(Color::RGB(125,125,125)),
                    _ => (),
                }

                if result > 0 {
                    let x_pos = x * 4 + x_offset;
                    let y_pos = y * 4 + y_offset;
                    let rect = Rect::new(x_pos, y_pos, PIXEL_SIZE, PIXEL_SIZE);
                    renderer.fill_rect(rect).unwrap();
                    // renderer.draw_point(Point::new(j as i32, i as i32)).unwrap();
                    // println!("drawing at {},{}", i + 10, j + 10);
                }
                print!("{}", result);
            }
            println!("");
        }
        println!("");
        let rect = Rect::new(x_offset, y_offset, RECT_SIZE, RECT_SIZE);
        renderer.set_draw_color(Color::RGB(0,0,0));
        renderer.draw_rect(rect).unwrap();

    }

    renderer.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
    }
}
