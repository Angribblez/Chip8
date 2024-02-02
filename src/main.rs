//use declarations
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::env;
use std::fs;

//main method
fn main() -> Result<(), String> {
    //graphics and input setup
    let sdl_context:sdl2::Sdl = sdl2::init()?;
    let video_subsystem:sdl2::VideoSubsystem = sdl_context.video()?;
    let window:sdl2::video::Window = video_subsystem.window("Chip 8 Emulator", 1280, 640)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let mut canvas:sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().build()
        .expect("could not make a canvas");
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    //emulated memory setup
    let mut memory:[u8; 0x1000] = [0; 0x1000];
    let mut display:[u8; 0x800] = [0; 0x800];
    let mut registers:[u8; 0x10] = [0; 0x10];
    let mut program_counter:usize = 0x200;
    let mut index_register:u16 = 0x0;

    //font setup
    memory[..0x50].clone_from_slice(&[
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ]);

    //get file name and load file
    let args:Vec<String> = env::args().collect();
    let file_name:&String = &args[1];
    let contents:Vec<u8> = fs::read(file_name).expect("");
    for i in 0..contents.len() {
        memory[program_counter + i] = contents[i];
    }

    //opcode loop
    let mut should_run:bool = true;
    while should_run {
        //input handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    should_run = false;
                },
                _ => {}
            }
        }

        //load 2 byte opcode
        let opcode = ((memory[program_counter] as u16) << 8) | (memory[program_counter+1] as u16);

        //check opcode and execute
        if opcode == 0x00E0 {
            //clear screen (00E0)
            //clear
            for i in 0..0xFF {
                display[i as usize] = 0x00;
            }
        }else if (opcode & 0xF000) == 0x1000 {
            //jump (1NNN)
            program_counter = ((opcode & 0x0FFF) - 2) as usize;
        }else if (opcode & 0xF000) == 0x6000 {
            //set register vx (6XNN)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //get value to set register to
            let reg_val:u8 = (opcode & 0x00FF) as u8;

            //update register
            registers[reg_num] = reg_val;
        }else if (opcode & 0xF000) == 0x7000 {
            //add value to register vx (7XNN)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //get value to set register to
            let reg_val:u8 = (opcode & 0x00FF) as u8;

            //update register
            registers[reg_num] += reg_val;
        }else if (opcode & 0xF000) == 0xA000 {
            //set index register I (ANNN)
            //get value to set register to and update register
            index_register = opcode & 0x0FFF;
        }else if (opcode & 0xF000) == 0xD000 {
            //display / draw (DXYN)
            //get coordinates and height
            let x:u8 = registers[((opcode & 0x0F00) >> 0x8) as usize] % 64;
            let y:u8 = registers[((opcode & 0x00F0) >> 0x4) as usize] % 32;
            let h:u8 = (opcode & 0x000F) as u8;

            //set VF to 0
            registers[0xF] = 0x0;

            //draw to screen
            //iterate through each row to draw
            for n in 0..h {
                //get row of sprite data from memory at I
                let row:u8 = memory[(index_register + (n as u16)) as usize];

                //loop through each pixel in byte
                for i in 0..8 {
                    //check if pixel is to be toggled
                    if row & (1<<(7-i)) != 0 {
                        //get screen memory address
                        let location:usize = ((y as usize) + (n as usize))*64 + (x as usize) + (i as usize);
                        //toggle pixel
                        display[location] += 1;
                        if display[location] == 2 {
                            display[location] = 0;
                        }
                    }
                    
                    //check if horizontal edge reached, if so go to next line
                    if x + i == 63 {
                        break;
                    }
                }

                //check if vertical edge reached, if so stop drawing
                if y + n == 31 {
                    break;
                }
            }
        }

        //increment program counter
        program_counter += 2;

        //redraw screen
        //clear canvas as black, and set color to white
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        //loop through each pixel line
        for y in 0..(32 as usize) {
            //loop through each pixel in line and draw
            for x in 0..(64 as usize) {
                //get pixel value and draw
                let pixel:u8 = display[y*64 + x];
                if pixel > 0 {
                    let _= canvas.fill_rect(Rect::new((x * 20) as i32, (y * 20) as i32, 20, 20));
                }
            }
        }

        //canvas redisplay and sleep
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 2_000_000_u32));
    }

    Ok(())
}