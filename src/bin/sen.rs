extern crate sen;
extern crate sdl2;
extern crate time;

use std::env;
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
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);

    // FIXME: God this is ugly. I really need to figure out ownership better :/
    let mut file = File::open(path).unwrap();
    let cartridge = Cartridge::load(&mut file);
    let mut file2 = File::open(path).unwrap();
    let cartridge2 = Cartridge::load(&mut file2);

    print!("Loaded ROM at {:?}", path);
    println!(" - {}", cartridge.header);

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

    let mut previous_time = time::precise_time_s();

    'running: loop {
        cpu.step();
        let ppu_result = cpu.ram.ppu.step(cpu.cycle);
        cpu.cycle = 0;

        if ppu_result.nmi { cpu.nmi(); }

        if ppu_result.new_frame {
            let t = time::precise_time_s();
            if t > previous_time + 1 as f64 {
                println!("{} FPS", cpu.ram.ppu.frames);
                previous_time = t;
                cpu.ram.ppu.frames = 0;
            }

            texture.update(None, &cpu.ram.ppu.frame_content, 256 * 3).unwrap();
            renderer.clear();
            renderer.copy(&texture, None, None);
            renderer.present();

            // FIXME: The whole controller thing doesn't work at all
            let keys: Vec<Keycode> = event_pump
                .keyboard_state()
                .pressed_scancodes()
                .filter_map(Keycode::from_scancode)
                .collect();

            cpu.ram.controller.buttons = keys;

            while let Some(event) = event_pump.poll_event() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => ()
                }
            }
        }
    }
}
