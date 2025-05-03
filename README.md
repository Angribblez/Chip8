# Chip8 Emulator
An emulator for Chip8 built in rust with slight timer modifications

## How to run
```$ cargo run -- rom_name.rom```

## Tools
This emulator was programmed in Rust using SDL2

## Features
- All features of original Chip 8 program

- Font data stored in memory space of original program

- Modified timers to operate at 62.5hz instead of 60hz for easier use. Timer runs 1/8th the speed of the 500hz CPU.

## FAQ
### Why was this emulator created?
This emulator was made to teach myself rust and emulation concepts in a fun yet difficult project.

### Why did you change the clock rate of the timers?
I changed the clock rate of the timers because it made the timers way easier to program, and has no noticeable effect on the timing of games.