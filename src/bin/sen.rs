extern crate sen;
extern crate sdl2;

use std::fs::File;
use std::path::Path;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

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

    let ppu = Ppu::new(cartridge2);
    let memory = CpuMemory::new(cartridge, ppu);
    let mut cpu = Cpu::new(memory);

    cpu.reset();
    cpu.ram.ppu.reset();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("wat", 256, 240)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().accelerated().build().unwrap();

    // renderer.set_draw_color(Color::RGB(0,0,0));
    renderer.clear();
    // renderer.set_draw_color(Color::RGB(255,255,255));

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut texture = renderer.create_texture_target(PixelFormatEnum::BGR24, 256, 240).unwrap();

    'running: loop {
        cpu.step();
        let ppu_result = cpu.ram.ppu.step(cpu.cycle);

        if ppu_result.nmi { cpu.nmi(); }

        if ppu_result.new_frame {
        //     // println!("Rendered new frame (CPU: {} PPU: {})", cpu.cycle, cpu.ram.ppu.cycle);
            texture.update(None, &cpu.ram.ppu.frame_content, 256 * 3).unwrap();
            renderer.clear();
            renderer.copy(&texture, None, None); //Some(Rect::new(0, 0, 256, 240)));
            renderer.present();
        }

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

// const PIXEL_SIZE: u32 = 4; // const RECT_SIZE: u32 = 32;
// const GRID_SIZE: i32 = 24;
//
// for n in 0..128 {
//     let x_offset: i32 = ((n % GRID_SIZE) * RECT_SIZE as i32) + PIXEL_SIZE as i32;//n * RECT_SIZE as i32 + 4;
//     let y_offset: i32 = (n / GRID_SIZE * RECT_SIZE as i32) + PIXEL_SIZE as i32;
//
//     for y in 0..8 {
//         let plane0 = ppu.vram_load(y as u16 + (n as u16 * 16));
//         let plane1 = ppu.vram_load(y as u16 + (n as u16 * 16) + 8);
//
//         for x in 0..8 {
//             let bit0 = (plane0 >> ((7 - ((x % 8) as u8)) as usize)) & 1;
//             let bit1 = (plane1 >> ((7 - ((x % 8) as u8)) as usize)) & 1;
//             let result = (bit1 << 1) | bit0;
//
//             match result {
//                 1 => renderer.set_draw_color(Color::RGB(215,215,215)),
//                 2 => renderer.set_draw_color(Color::RGB(190,190,190)),
//                 3 => renderer.set_draw_color(Color::RGB(125,125,125)),
//                 _ => (),
//             }
//
//             if result > 0 {
//                 let x_pos = x * 4 + x_offset;
//                 let y_pos = y * 4 + y_offset;
//                 let rect = Rect::new(x_pos, y_pos, PIXEL_SIZE, PIXEL_SIZE);
//                 renderer.fill_rect(rect).unwrap();
//             }
//             print!("{}", result);
//         }
//         println!("");
//     }
//     println!("");
//     let rect = Rect::new(x_offset, y_offset, RECT_SIZE, RECT_SIZE);
//     renderer.set_draw_color(Color::RGB(0,0,0));
//     renderer.draw_rect(rect).unwrap();
//
// }
//
// renderer.present();
