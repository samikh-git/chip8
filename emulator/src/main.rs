use rand::prelude::*;
use std::{thread, time::Duration};
use minifb::{Key, Window, WindowOptions};

mod file_utils;

const CLOCK_SPEED: f64 = 1.0/1000.0;

struct DelayTimer {
    timer: u8
}

impl DelayTimer {
    fn new(length: u8) -> Self {
        Self { timer: length }
    }

    fn set(&mut self, x: u8) {
        self.timer = x;
    }

    fn get(&self) -> u8 {
        self.timer
    }

    fn tick(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
        }
    }

    fn is_empty(&self) -> bool {
        self.timer == 0
    }
}     

struct CHIP8 {
    memory: [u8; 4096],
    program_counter: usize,
    index_register: usize,

    flag_register: [u8; 16],
    
    
    stack: [u16; 16],
    stack_pointer: usize,

    delay_timer: DelayTimer,
    sound_timer: DelayTimer,

    display: [[bool; 64]; 32],
    key_buffer: [bool; 16]
}

impl CHIP8 {
    fn new() -> Self {
        Self {
            memory: [0; 4096],
            program_counter: 0x200, 
            index_register: 0,
            flag_register: [0; 16],
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: DelayTimer::new(0),
            sound_timer: DelayTimer::new(0),
            display: [[false; 64]; 32],
            key_buffer: [false; 16]
        }
    }

    fn push(&mut self, value: u16) {
        if self.stack_pointer < 15 {
            self.stack[self.stack_pointer] = value;
            self.stack_pointer += 1;
        } else {
            panic!("Stack Overflow")
        }
    }

    fn pop(&mut self) -> u16 {
        if self.stack_pointer > 0 {
            self.stack_pointer -= 1; 
            self.stack[self.stack_pointer]
        } else {
            panic!("Stack Underflow")
        }
    }

    fn load_instruction(&mut self) -> u16 {
        let high_byte = self.memory[self.program_counter];
        let low_byte = self.memory[self.program_counter + 1];
        self.program_counter += 2;
        ((high_byte as u16) << 8) | (low_byte as u16)
    }

    fn clear(&mut self) {
        self.display = [[false; 64]; 32];
    }

    fn jump(&mut self, nnn: usize) {
        self.program_counter = nnn;
    }

    fn call(&mut self, nnn: usize) {
        self.push(self.program_counter as u16);
        self.program_counter = nnn;
    }

    fn return_routine(&mut self) {
        self.program_counter = self.pop() as usize;
    } 

    fn skip_conditionally3(&mut self, x: usize, nn: u8) {
        if self.flag_register[x] == nn {
            self.program_counter += 2;
        }
    }

    fn skip_conditionally4(&mut self, x: usize, nn: u8) {
        if self.flag_register[x] != nn {
            self.program_counter += 2;
        }
    }

    fn skip_conditionally5(&mut self, x: usize, y: usize) {
        if self.flag_register[x] == self.flag_register[y] {
            self.program_counter += 2;
        }
    }

    fn skip_conditionally9(&mut self, x: usize, y: usize) {
        if self.flag_register[x] != self.flag_register[y] {
            self.program_counter += 2;
        }
    }

    fn set6(&mut self, x: usize, nn: u8) {
        self.flag_register[x] = nn;
    }

    fn add7(&mut self, x: usize, nn: u8) {
        self.flag_register[x] = self.flag_register[x].wrapping_add(nn);
    }

    fn set8(&mut self, x: usize, y: usize) {
        self.flag_register[x] = self.flag_register[y];
    }

    fn bitor(&mut self, x: usize, y: usize) {
        let operation = self.flag_register[x] | self.flag_register[y];
        self.flag_register[x] = operation;
    }

    fn bitand(&mut self, x: usize, y: usize) {
        let operation = self.flag_register[x] & self.flag_register[y];
        self.flag_register[x] = operation;
    }

    fn bitxor(&mut self, x: usize, y: usize) {
        let operation = self.flag_register[x] ^ self.flag_register[y];
        self.flag_register[x] = operation;
    }

    fn add8(&mut self, x: usize, y: usize) {
        let sum = (self.flag_register[x] as u16) + (self.flag_register[y] as u16);
        if sum > 255 {
            self.flag_register[0xF] = 1;
        } else {
            self.flag_register[0xF] = 0;
        }
        self.flag_register[x] = sum as u8;
    }

    fn subtract(&mut self, x: usize, y: usize, left_right: bool) {
        let left;
        let right;
        
        if left_right {
            left = x;
            right = y;
        } else {
            left = y;
            right = x;
        }

        let left_operand = self.flag_register[left];
        let right_operand = self.flag_register[right];

        if left_operand > right_operand {
            self.flag_register[0xF] = 1;
        } else {
            self.flag_register[0xF] = 0;
        }

        self.flag_register[x] = left_operand.wrapping_sub(right_operand);
    }

    fn shift(&mut self, x: usize, _y: usize, left: bool) { 
        let val = self.flag_register[x];
        if left {
            self.flag_register[0xF] = (val & 0x80) >> 7; 
            self.flag_register[x] = val << 1;
        } else {
            self.flag_register[0xF] = val & 0x01;
            self.flag_register[x] = val >> 1;
        }
    }

    fn jump_offset(&mut self, nnn: usize) {
        self.program_counter = nnn + self.flag_register[0] as usize;
    }

    fn random(&mut self, x: usize, nn: u8) {
        let mut rng = rand::rng();
        let random_u8 = rng.random::<u8>();
        self.flag_register[x] = nn & random_u8;
    }

    fn draw(&mut self, x: usize, y: usize, n: u8) {
        self.flag_register[0xF] = 0;

        for row in 0..n {
            let y_pos = (self.flag_register[y] as usize + row as usize) % 32;
            let sprite_byte = self.memory[(self.index_register + row as usize) as usize];

            for col in 0..8 {
                let x_pos = (self.flag_register[x] as usize + col) % 64;
                
                let sprite_pixel = (sprite_byte >> (7 - col)) & 1;

                if sprite_pixel == 1 {
                    // Accessing [Y][X] to match [[bool; 64]; 32]
                    if self.display[y_pos][x_pos] == true {
                        self.flag_register[0xF] = 1;
                    }

                    self.display[y_pos][x_pos] ^= true;
                }
            }
        }
    }

    fn skip_key(&mut self, x: usize, should_be_pressed: bool) {
        let key = self.flag_register[x] as usize;
        let is_currently_pressed = self.key_buffer[key];

        if is_currently_pressed == should_be_pressed {
            self.program_counter += 2;
        }
    }

    fn absorb_timer(&mut self, x: usize) {
        self.flag_register[x] = self.delay_timer.get();
    }

    fn set_timer(&mut self, x: usize, delay: bool) {
        if delay {
            self.delay_timer.set(self.flag_register[x]);
        } else {
            self.sound_timer.set(self.flag_register[x]);
        }
    }

    fn add_to_index(&mut self, x: usize) {
        self.index_register += self.flag_register[x] as usize;
    }

    fn convert_decimal(&mut self, x: usize) {
        let number = self.flag_register[x];
        let hundreth = (number / 100) % 10;
        let tenth = (number / 10) % 10;
        let unit = number % 10;
        self.memory[self.index_register] = hundreth;
        self.memory[self.index_register + 1] = tenth;
        self.memory[self.index_register + 2] = unit;
    }

    fn store(&mut self, x: usize) {
        for i in 0..x+1 {
            self.memory[self.index_register + i] = self.flag_register[i];
        }
    }

    fn load(&mut self, x: usize) {
        for i in 0..x+1 {
            self.flag_register[i] = self.memory[self.index_register + i];
        }
    }

    fn get_font_character(&mut self, x: usize) {
        let character = self.flag_register[x] as usize;
        self.index_register = (0x50 + (character * 5)) as usize;
    }

    fn wait_for_key(&mut self, x: usize) {
        let mut pressed = false;
        for i in 0..self.key_buffer.len() {
            if self.key_buffer[i] {
                self.flag_register[x] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            self.program_counter -= 2;
        }
    }

    pub fn load_rom_into_memory(&mut self, rom_data: &[u8]) {
        let start_address = 0x200;
        
        for (i, &byte) in rom_data.iter().enumerate() {
            if start_address + i < 4096 {
                self.memory[start_address + i] = byte;
            } else {
                println!("Warning: ROM is too big for memory!");
                break;
            }
        }
    }

    fn execute(&mut self, instruction: u16) {
        let nibble1 = (instruction & 0xF000) >> 12; 
        let x = (instruction & 0x0F00) >> 8;  
        let y = (instruction & 0x00F0) >> 4;  
        let n = instruction & 0x000F;       
        let nn = instruction & 0x00FF;       
        let nnn = instruction & 0x0FFF;

        match nibble1 {
            0x0 => match nnn {
                0x0E0 => self.clear(),
                0x0EE => self.return_routine(),
                _ => panic!("Unknown opcode: {:X}", instruction)
            },
            0x1 => self.jump(nnn as usize),
            0x2 => self.call(nnn as usize),
            0x3 => self.skip_conditionally3(x as usize, nn as u8),
            0x4 => self.skip_conditionally4(x as usize, nn as u8),
            0x5 => self.skip_conditionally5(x as usize, y as usize),
            0x6 => self.set6(x as usize, nn as u8),
            0x7 => self.add7(x as usize, nn as u8),
            0x8 => match n {
                0 => self.set8(x as usize, y as usize),
                1 => self.bitor(x as usize, y as usize),
                2 => self.bitand(x as usize, y as usize),
                3 => self.bitxor(x as usize, y as usize),
                4 => self.add8(x as usize, y as usize), 
                5 => self.subtract(x as usize, y as usize, true),
                6 => self.shift(x as usize, y as usize, false),
                7 => self.subtract(x as usize, y as usize, false),
                0xE => self.shift(x as usize, y as usize, true),
                _ => panic!("Unknown opcode")
            },
            0x9 => self.skip_conditionally9(x as usize, y as usize),
            0xA => {self.index_register = nnn as usize},
            0xB => self.jump_offset(nnn as usize),
            0xC => self.random(x as usize, nn as u8),
            0xD => self.draw(x as usize, y as usize, n as u8),
            0xE => match nn {
                0x9E => self.skip_key(x as usize, true),
                0xA1 => self.skip_key(x as usize, false),
                _ => panic!("Unknown upcode")
            },
            0xF => match nn {
                0x07 => self.absorb_timer(x as usize),
                0x15 => self.set_timer(x as usize, true),
                0x18 => self.set_timer(x as usize, false),
                0x1E => self.add_to_index(x as usize),
                0x0A => self.wait_for_key(x as usize),
                0x29 => self.get_font_character(x as usize),
                0x33 => self.convert_decimal(x as usize),
                0x55 => self.store(x as usize),
                0x65 => self.load(x as usize),
                _ => panic!("Unknown opcode")
            },
            _ => panic!("Unknown opcode"),
        }         
    }
}

fn main() {
    let mut window = Window::new(
        "Rust CHIP-8",
        64, 32,
        WindowOptions { scale: minifb::Scale::X16, ..WindowOptions::default() }
    ).unwrap();

    let mut chip8 = CHIP8::new();

    // 1. Load Font
    let font_bytes = file_utils::load_font_from_text("/Users/sami/CHIP8-emulator/emulator/assets/font.txt");
    for (i, x) in font_bytes.iter().enumerate() {
        chip8.memory[0x50 + i] = *x; // Load at 0x50 to match your FX29 logic
    }

    // 2. Load ROM (Replace with your actual .ch8 file path)
    let rom_bytes = file_utils::load_rom("/Users/sami/CHIP8-emulator/emulator/assets/coin_flip.ch8");
    chip8.load_rom_into_memory(&rom_bytes);

    // 3. The Main Loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update Keys
        let keys = window.get_keys();
        chip8.key_buffer = [false; 16];
        for key in keys {
            if let Some(val) = file_utils::map_key(key) {
                chip8.key_buffer[val] = true;
            }
        }

        // Run CPU cycles (~500Hz - 700Hz)
        // Since window update is 60Hz, 10 cycles per frame = 600Hz
        for _ in 0..10 {
            let instruction = chip8.load_instruction();
            chip8.execute(instruction);
        }

        // Update Timers at 60Hz
        chip8.delay_timer.tick();
        chip8.sound_timer.tick();

        // Update Screen at 60Hz
        let buffer: Vec<u32> = chip8.display.iter().flatten()
            .map(|&p| if p { 0xFFFFFF } else { 0x000000 })
            .collect();

        window.update_with_buffer(&buffer, 64, 32).unwrap();
    }
}

