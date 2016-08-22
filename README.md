# SEN - Yet another NES emulator

Why? Because I can. And to learn Rust.
This is heavily WIP right now and completely unusable. Here be dragons.
Most of the code is messy and/or commented. Please don't mind me :)

To test, just run `cargo run --release`.

## What works

* Can start Donkey Kong
* CPU and memory
* Palettes
* Basic PPU

## TODO

* Load any ROM (only work with Donkey Kong right now)
* CLI arguments
* Controllers (there's some code but it's broken)
* Fix the very glitchy PPU
* 8x16 Tiles
* Scrolling background
* More memory mappers (this will require some refactoring of the memory interface)
* Implement the 6502 glitch
* Test the CPU