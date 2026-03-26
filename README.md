# CHIP8 Emulator in Rust

This is basic CHIP8 Emulator built in Rust. I mainly followed this guide: [https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#fx29-font-character](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#fx29-font-character).

There are two main files: `main.rs` and `file_utils.rs`. `main.rs` holds the CHIP8 abstraction and instruction handling. `file_utils.rs` holds utils to for loading into ROM and for loading the fonts.

I need to add test suites to this file. So, this will be coming soon. 

This project was mainly an opportunity to learn more about emulators and the Rust language.