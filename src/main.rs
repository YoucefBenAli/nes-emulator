mod AddressingModes;
mod OpCodes;
mod CPU;
mod bus;
mod memory;
mod rom;
mod ppu;

use rom::Rom;
use bus::Bus;

use std::{env, fs::File};
use std::io::Read;
use event::Event;
use keyboard::Keycode;
use pixels::{Color, PixelFormatEnum};
use rand::Rng;
use sdl2::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() == 1 { // Cargo run always includes the executable name as an argument
        snake_game();
    } else {
        let file_name: &String = args.get(1).unwrap();
        run_rom(file_name);
    }
}

fn run_rom(file_name: &String) {
    let cartridge: Rom = Rom::new(&get_file_as_byte_vec(file_name)).unwrap();
    let bus: Bus = Bus::new(cartridge);

    let mut cpu: CPU::CPU = CPU::CPU::new(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;
    cpu.run_with_callback(move |cpu| {
        println!("{}", cpu.trace());
    })
}

fn snake_game() {
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
       .create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();

    let mut screen_state = [0 as u8; 32 * 3 * 32];
    let mut rng = rand::rng();

    let cartridge: Rom = Rom::new(&get_file_as_byte_vec(&"snake.nes".to_string())).unwrap();
    let bus: Bus = Bus::new(cartridge);

    let mut cpu: CPU::CPU = CPU::CPU::new(bus);
    cpu.reset();

    cpu.run_with_callback(move |cpu| {
        handle_user_input(cpu, &mut event_pump);
        cpu.mem_write(0xfe, rng.random_range(1..16));

        if read_screen_state(cpu, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }

        ::std::thread::sleep(std::time::Duration::new(0, 70000));
    });


}

fn handle_user_input(cpu: &mut CPU::CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0)
            },
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                cpu.mem_write(0xff, 0x77);
            },
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                cpu.mem_write(0xff, 0x73);
            },
            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                cpu.mem_write(0xff, 0x61);
            },
            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                cpu.mem_write(0xff, 0x64);
            }
            _ => {}
        }
    }
 }

 fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
 }
 
 fn read_screen_state(cpu: &CPU::CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
 }
 
fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    std::fs::read(filename).unwrap()
}