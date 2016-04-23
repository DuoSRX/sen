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
use sen::controller::Controller;
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
    let controller = Controller::new();
    let memory = CpuMemory::new(cartridge, ppu, controller);
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

    renderer.clear();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut texture = renderer.create_texture_target(PixelFormatEnum::BGR24, 256, 240).unwrap();

    'running: loop {
        cpu.step();
        let ppu_result = cpu.ram.ppu.step(cpu.cycle);

        if ppu_result.nmi { cpu.nmi(); }

        if ppu_result.new_frame {
            texture.update(None, &cpu.ram.ppu.frame_content, 256 * 3).unwrap();
            renderer.clear();
            renderer.copy(&texture, None, None); //Some(Rect::new(0, 0, 256, 240)));
            renderer.present();

            while let Some(event) = event_pump.poll_event() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    }
                    _ => ()
                }
            }

        }

        // for event in event_pump.poll_iter() {
        //     match event {
        //         Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
        //             break 'running
        //         },
        //         Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
        //             cpu.ram.controller.buttons[0] = true;
        //         }
        //         Event::KeyDown { keycode: Some(Keycode::X), .. } => {
        //             cpu.ram.controller.buttons[1] = true;
        //         }
        //         Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
        //             cpu.ram.controller.buttons[3] = true;
        //         }
        //         Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
        //             cpu.ram.controller.buttons[4] = true;
        //         }
        //         Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
        //             cpu.ram.controller.buttons[5] = true;
        //         }
        //         Event::KeyUp { keycode: Some(_key), .. } => {
        //             cpu.ram.controller.buttons = [false; 8];
        //         }
        //         _ => {}
        //     }
        // }
    }
}
