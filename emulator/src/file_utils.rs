use std::fs;
use minifb::{Key};
use std::io;


pub fn load_font_from_text(filename: &str) -> Vec<u8> {
    let content = fs::read_to_string(filename)
        .expect("Could not read font file");

    content
        .lines()
        .map(|line| line.trim())               
        .filter(|line| !line.is_empty())       
        .map(|line| {
            let hex_str = line.strip_prefix("0x").unwrap_or(line);
            u8::from_str_radix(hex_str, 16).expect("Invalid hex value in font file")
        })
        .collect()
}

pub fn map_key(key: Key) -> Option<usize> {
    match key {
        Key::Key1 => Some(0x1), Key::Key2 => Some(0x2), Key::Key3 => Some(0x3), Key::Key4 => Some(0xC),
        Key::Q    => Some(0x4), Key::W    => Some(0x5), Key::E    => Some(0x6), Key::R    => Some(0xD),
        Key::A    => Some(0x7), Key::S    => Some(0x8), Key::D    => Some(0x9), Key::F    => Some(0xE),
        Key::Z    => Some(0xA), Key::X    => Some(0x0), Key::C    => Some(0xB), Key::V    => Some(0xF),
        _ => None,
    }
}

pub fn load_rom(path: &str) -> Vec<u8> {
    fs::read(path).expect("Could not find ROM file! Check the path.")
}