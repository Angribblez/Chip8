use sdl2::audio::AudioSpecDesired;
use sdl2::keyboard::KeyboardState;
//use declarations
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::AudioSubsystem;
use sdl2::EventPump;
use std::time::Duration;
use std::env;
use std::fs;
use rand::Rng;

//structs and methods
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl sdl2::audio::AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

//main method
fn main() -> Result<(), String> {
    //graphics, sound, and input setup
    let sdl_context:sdl2::Sdl = sdl2::init()?;
    let audio_subsystem:AudioSubsystem = sdl_context.audio()?;
    let video_subsystem:sdl2::VideoSubsystem = sdl_context.video()?;
    let window:sdl2::video::Window = video_subsystem.window("Chip 8 Emulator", 1280, 640)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");
    let mut canvas:sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().build()
        .expect("could not make a canvas");
    canvas.clear();
    canvas.present();
    let mut event_pump:EventPump = sdl_context.event_pump()?;
    let desired_spec:AudioSpecDesired = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };
    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        // initialize the audio callback
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    }).unwrap();

    //emulated memory setup
    let mut memory:[u8; 0x1000] = [0; 0x1000];
    let mut display:[u8; 0x800] = [0; 0x800];
    let mut registers:[u8; 0x10] = [0; 0x10];
    let mut program_counter:usize = 0x200;
    let mut index_register:u16 = 0x0;
    let mut stack:[usize; 0xFF] = [0; 0xFF];
    let mut stack_index:usize = 0x0;
    let mut delay_timer:u8 = 0xFF;
    let mut sound_timer:u8 = 0xFF;
    let mut input:[u8; 0x10] = [0; 0x10];

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
    let mut timer_counter:u8 = 0;
    while should_run {
        //input handling
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    should_run = false;
                },

                //check keypad inputs and store to array
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    input[1] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Num1), .. } => {
                    input[1] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    input[2] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Num2), .. } => {
                    input[2] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                    input[3] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Num3), .. } => {
                    input[3] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                    input[0xC] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Num4), .. } => {
                    input[0xC] = 0;
                },

                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    input[4] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Q), .. } => {
                    input[4] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    input[5] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::W), .. } => {
                    input[5] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::E), .. } => {
                    input[6] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::E), .. } => {
                    input[6] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    input[0xD] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::R), .. } => {
                    input[0xD] = 0;
                },

                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    input[7] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::A), .. } => {
                    input[7] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    input[8] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                    input[8] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    input[9] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::D), .. } => {
                    input[9] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    input[0xE] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::F), .. } => {
                    input[0xE] = 0;
                },

                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    input[0xA] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::Z), .. } => {
                    input[0xA] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                    input[0] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::X), .. } => {
                    input[0] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    input[0xB] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::C), .. } => {
                    input[0xB] = 0;
                },
                Event::KeyDown { keycode: Some(Keycode::V), .. } => {
                    input[0xF] = 1;
                },
                Event::KeyUp { keycode: Some(Keycode::V), .. } => {
                    input[0xF] = 0;
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
        }else if opcode == 0x00EE{
            //return from subroutine
            //set program counter to value stored in highest part of stack
            program_counter = stack[stack_index-1];
        }else if (opcode & 0xF000) == 0x1000 {
            //jump (1NNN)
            program_counter = ((opcode & 0x0FFF) - 2) as usize;
        }else if (opcode & 0xF000) == 0x2000 {
            //call subroutine (2NNN)
            //set highest part of stack to program counter
            stack[stack_index] = program_counter;
            stack_index = stack_index.wrapping_add(1);

            //set program counter equal to passed value
            program_counter = ((opcode & 0x0FFF) - 2) as usize;
        }else if (opcode & 0xF000) == 0x3000 {
            //instruction skip (3XNN) if Vx = NN
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //get value to compare register to
            let cmp_val:u8 = (opcode & 0x00FF) as u8;

            //compare
            if registers[reg_num] == cmp_val {
                program_counter = program_counter.wrapping_add(2);
            }
        }else if (opcode & 0xF000) == 0x4000 {
            //instruction skip (4XNN) if Vx != NN
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //get value to compare register to
            let cmp_val:u8 = (opcode & 0x00FF) as u8;

            //compare
            if registers[reg_num] != cmp_val {
                program_counter = program_counter.wrapping_add(2);
            }
        }else if (opcode & 0xF00F) == 0x5000 {
            //instruction skip (5XY0) if Vx = Vy
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //compare
            if registers[reg_num_x] == registers[reg_num_y] {
                program_counter = program_counter.wrapping_add(2);
            }
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
            registers[reg_num] = registers[reg_num].wrapping_add(reg_val);
        }else if (opcode & 0xF00F) == 0x8000 {
            //set value of Vx to value of Vy (8XY0)
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] = registers[reg_num_y];
        }else if (opcode & 0xF00F) == 0x8001 {
            //set value of Vx to value of Vx or Vy (8XY1)
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] |= registers[reg_num_y];
        }else if (opcode & 0xF00F) == 0x8002 {
            //set value of Vx to value of Vx and Vy (8XY2)
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] &= registers[reg_num_y];
        }else if (opcode & 0xF00F) == 0x8003 {
            //set value of Vx to value of Vx xor Vy (8XY3)
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] ^= registers[reg_num_y];
        }else if (opcode & 0xF00F) == 0x8004 {
            //set value of Vx to value of Vx + Vy (8XY4), set VF to whether or not there was an overflow
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //check for overflow
            if  registers[reg_num_x].checked_add(registers[reg_num_y]) == None {
                registers[0xF] = 1;
            }else{
                registers[0xF] = 0;
            }

            //update register
            registers[reg_num_x] = registers[reg_num_x].wrapping_add(registers[reg_num_y]);
        }else if (opcode & 0xF00F) == 0x8005 {
            //set value of Vx to value of Vx - Vy (8XY5), set VF to whether or not there was an underflow
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //check for overflow
            if  registers[reg_num_x].checked_sub(registers[reg_num_y]) == None {
                registers[0xF] = 0;
            }else{
                registers[0xF] = 1;
            }

            //update register
            registers[reg_num_x] -= registers[reg_num_y];
        }else if (opcode & 0xF00F) == 0x8006 {
            //set value of Vx to value of Vy shifted 1 bit to the right (8XY6), set Vf to the shifted bit
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] = registers[reg_num_y];
            registers[0xF] = registers[reg_num_x] % 2;
            registers[reg_num_x] = registers[reg_num_x] >> 1;
        }else if (opcode & 0xF00F) == 0x8007 {
            //set value of Vx to value of Vy - Vx (8XY7), set VF to whether or not there was an underflow
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //check for overflow
            if  registers[reg_num_y].checked_sub(registers[reg_num_x]) == None {
                registers[0xF] = 0;
            }else{
                registers[0xF] = 1;
            }

            //update register
            registers[reg_num_x] = registers[reg_num_y] - registers[reg_num_x];
        }else if (opcode & 0xF00F) == 0x800E {
            //set value of Vx to value of Vy shifted 7 bits to the right (8XYE), set Vf to the shifted bit
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            registers[reg_num_x] = registers[reg_num_y];
            registers[0xF] = registers[reg_num_x] >> 7;
            registers[reg_num_x] = registers[reg_num_x] << 1;
        }else if (opcode & 0xF00F) == 0x9000 {
            //skip next instruction if Vx != Vy (9XY0)
            //get register numbers
            let reg_num_x:usize = ((opcode & 0x0F00) >> 0x8) as usize;
            let reg_num_y:usize = ((opcode & 0x00F0) >> 0x4) as usize;

            //update register
            if registers[reg_num_x] != registers[reg_num_y] {
                program_counter = program_counter.wrapping_add(2);
            }
        }else if (opcode & 0xF000) == 0xA000 {
            //set index register I (ANNN)
            //get value to set register to and update register
            index_register = opcode & 0x0FFF;
        }else if (opcode & 0xF000) == 0xB000 {
            //jump to NNN + V0 (1NNN)
            program_counter = ((opcode & 0x0FFF) + (registers[0] as u16) - 2) as usize;
        }else if (opcode & 0xF000) == 0xC000 {
            //set value of Vx to random & NN (CXNN)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //get value to bitwise and random value with
            let val:u8 = (opcode & 0x00FF) as u8;

            //update register
            registers[reg_num] = rand::thread_rng().gen();
            registers[reg_num] &= val;
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
                        display[location] = display[location].wrapping_add(1);
                        if display[location] == 2 {
                            display[location] = 0;
                            registers[0xF] = 1;
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
        }else if (opcode & 0xF0FF) == 0xE09E {
            //skip next instruction if key in VX is pressed (EX9E)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //check register
            if registers[reg_num] < 0x10 {
                //check key
                if input[registers[reg_num] as usize] == 1 {
                    program_counter += 2;
                }
            }
        }else if (opcode & 0xF0FF) == 0xE0A1 {
            //skip next instruction if key in VX is not pressed (EXA1)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //check register
            if registers[reg_num] < 0x10 {
                //check key
                if input[registers[reg_num] as usize] == 0 {
                    program_counter += 2;
                }
            }  
        }else if (opcode & 0xF0FF) == 0xF007 {
            //set vx to delay timer (FX07)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update register
            registers[reg_num] = delay_timer;
        }else if (opcode & 0xF0FF) == 0xF00A {
            //await key press and store code in VX (FX0A)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //loop through inputs
            let mut i = 0;
            for key in input {
                //check key press
                if key == 1 {
                    registers[reg_num] = i;
                    program_counter += 2;
                    break;
                }
                i += 1;
            }

            //decrement program counter
            program_counter -= 2;
        }else if (opcode & 0xF0FF) == 0xF015 {
            //set delay timer to vx (FX15)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update timer
            delay_timer = registers[reg_num];
        }else if (opcode & 0xF0FF) == 0xF018 {
            //set sound timer to vx (FX18)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update timer
            sound_timer = registers[reg_num];
        }else if (opcode & 0xF0FF) == 0xF01E {
            //adds Vx to index register (FX1E)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update index register
            index_register += registers[reg_num] as u16;
        }else if (opcode & 0xF0FF) == 0xF029 {
            //sets index register to sprite address of char in Vx (FX29)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update index register
            index_register = ((registers[reg_num] & 0x0F) as u16) * 5;
            println!("{} {:#06x}", registers[reg_num], index_register);
        }else if (opcode & 0xF0FF) == 0xF033 {
            //store bcd representation of Vx in I, I+1, and I+2 (FX33)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update index register
            memory[(index_register) as usize] = registers[reg_num]/100;
            memory[(index_register + 1) as usize] = (registers[reg_num] - 100*(registers[reg_num]/100))/10;
            memory[(index_register + 2) as usize] = registers[reg_num] - 10*(registers[reg_num]/10);
        }else if (opcode & 0xF0FF) == 0xF055 {
            //store V0 to Vx in index register to index register + X (FX55)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update index register
            for i in 0..(reg_num + 1) {
                memory[(index_register as usize) + i] = registers[i];
            }
        }else if (opcode & 0xF0FF) == 0xF065 {
            //fill V0 to Vx from index register to index register + X (FX65)
            //get register number
            let reg_num:usize = ((opcode & 0x0F00) >> 0x8) as usize;

            //update index register
            for i in 0..(reg_num + 1) {
                registers[i] = memory[(index_register as usize) + i];
            }
        }else{
            println!("{:#06x}", opcode)
        }

        //increment program counter
        program_counter = program_counter.wrapping_add(2);

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

        //timer management
        if timer_counter >= 7 {
            if delay_timer > 0 {
                delay_timer -= 1;
            }
            if sound_timer > 0 {
                sound_timer -= 1;
            }
            timer_counter = 0;
        }else{
            timer_counter += 1;
        }

        //sound
        if sound_timer != 0 {
            device.resume();
        }else{
            device.pause();
        }

        //canvas redisplay and sleep
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 2_000_000_u32));
    }

    Ok(())
}