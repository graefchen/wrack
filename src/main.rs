use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::env;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub static FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 1
    0x20, 0x60, 0x20, 0x20, 0x70, // 2
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 3
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 4
    0x90, 0x90, 0xF0, 0x10, 0x10, // 5
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 6
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 7
    0xF0, 0x10, 0x20, 0x40, 0x40, // 8
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // A
    0xF0, 0x90, 0xF0, 0x90, 0x90, // B
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // C
    0xF0, 0x80, 0x80, 0x80, 0xF0, // S
    0xE0, 0x90, 0x90, 0x90, 0xE0, // E
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // F
    0xF0, 0x80, 0xF0, 0x80, 0x80  // G
];

#[derive(Debug, Clone, Copy)]
pub struct Display {
    pub memory: [u8; 2048],
}

impl Display {

    ///
    pub fn new() -> Display {
        Display { memory: [0; 2048] }
    }

    ///
    pub fn set_pixel(&mut self, x: usize, y: usize, on: bool) {
        self.memory[x + y * WIDTH] = on as u8;
    }

    ///
    pub fn get_pixel(&mut self, x: usize, y: usize) -> bool {
        self.memory[x + y * WIDTH] == 1
    }

    /// Clear Screen instruction
    pub fn cls(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.set_pixel(x, y, false);
            }
        }
    }

    /// Draw Screen instruction
    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        for j in 0..rows {
            let row = sprite[j];
            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;
                if new_value == 1 {
                    let xi = (x + i) % WIDTH;
                    let yj = (y + j) % HEIGHT;
                    let old_value = self.get_pixel(xi, yj);
                    if old_value {
                        collision = true;
                    }
                    self.set_pixel(xi, yj, (new_value == 1) ^ old_value);
                }
            }
        }
        return  collision;
    }
}

/// The Keypad implementation ...
/// The old one did go from 0 to F
/// But we shall use an easier
/// methode with:
/// 1 2 3 4
/// Q W E R
/// A S D F
/// Z X C V
/// Make it scan codes instead of strings
pub struct Keypad {

}

pub struct Cpu {
    // index register
    pub i: u16,
    // program counter
    pub pc: u16,
    // memory
    pub memory: [u8; 4096],
    // register
    pub v: [u8; 16],
    // peripherals
    // pub keypad: Keypad,
    pub display: Display,
    // stack
    pub stack: [u16; 16],
    // stack pointer
    pub sp: u8,
    // delay timer
    pub dt: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            i: 0,
            pc: 0,
            memory: [0; 4096],
            v: [0; 16],
            display: Display::new(),
            stack: [0; 16],
            sp: 0,
            dt: 0,
        }
    }

    pub fn reset(&mut self) {
        self.i = 0;
        // Historically the program was loaded in memory after the chip-8 interpreter
        // that is why it starts at 0x200 or 512
        self.pc = 0x200;
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.dt = 0;
        self.display.cls();
        // Load in the font
        for i in 0..80 {
            self.memory[i] = FONT_SET[i];
        }
    }

    pub fn load_rom(&mut self, file_path: String) -> io::Result<u16> {
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer)?;
        for i in 0..buffer.len() {
            self.memory[0x200 + i] = buffer[i];
            // println!("i is {i}");
            // println!("{:>08b}", self.memory[i]);
        }
        Ok(buffer.len() as u16)
    }

    pub fn execute_cycle(&mut self) {
        let opcode: u16 = read_word(self.memory, self.pc);
        self.process_opcode(opcode);
    }

    fn process_opcode(&mut self, opcode: u16) {
        // Extract the parameters
        // second nibble
        let x = ((opcode & 0x0F00) >> 8) as usize;
        // third nibble
        let y = ((opcode & 0x00F0) >> 4) as usize;
        // value at register x
        let vx = self.v[x];
        // value at register y
        let vy = self.v[y];
        // the second, third and forth nibbles
        let nnn = opcode & 0x0FFF;
        // the second byte(third and forth nibble)
        let kk = (opcode & 0x00FF) as u8;
        // fourt nibble
        let n = (opcode & 0x000F) as u8;

        // break up into nibbles (1/2 byte; 4 bit)
        // in bitform would it look lite this:
        // 1111111111111111
        // [1111] [1111] [1111] [1111]
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        // increment the counter
        self.pc += 2;

        // printline for help with opcodes
        // println!("{:X}, {:X}, {:X}, {:X}", op_1, op_2, op_3, op_4);

        // matching the opcodes...
        // with handwritten comments with much detail
        match (op_1, op_2, op_3, op_4) {
            // CLS
            // Clear the display
            (0, 0, 0xE, 0) => self.display.cls(),

            // RET
            // Return from a subroutine
            // Set the pc on the address of the top of the stack,
            // then decrement the stack by 1
            (0, 0, 0xE, 0xE) => todo!("'RET' not implemented yet!"),

            // JP addr
            // Jump to location addr
            (0x1, _, _, _) => self.pc = nnn,

            // CALL addr
            // Call subroutine at nnn
            // Increment the sp, the put current PC on the top of the stack,
            // Then set the PC to nnn
            (0x2, _, _, _) => todo!("CALL addr"),

            // SE vx, byte"
            // Skip next instruction if Vx = kk
            // Compare register Vk to kk and
            // if they are equal increment pc by 2
            (0x3, _, _, _) => todo!("SE vx, byte"),

            // SNE vx, byte
            // Skip next instruction if Vx != kk
            // Compare register Vx to kk and
            // if they are not equal increment pc by 2
            (0x4, _, _, _) => todo!("SNE vx, byte"),

            // SE Vx, Vy
            // Skip next instruction if Vx = Vy.
            // Compares register Vx to register Vy and
            // if they are equal increment tpc by 2
            (0x5, _, _, _) => todo!("SE Vx, Vy"),

            // LD Vx, byte
            // Put value kk into register Vx
            (0x6, _, _, _) => self.v[x] = kk,

            // ADD Vx, byte
            // Add the value kk to the value of register Vx
            // and the store the result in Vx
            (0x7, _, _, _) => self.v[x] = self.v[x] + kk,

            // LD Vx, Vy
            // Stores the value of Vy in register Vx
            (0x8, _, _, 0x0) => todo!("LD Vx, Vy"),
            // OR Vx, Vy
            // Set Vx = Vx OR Vy
            // Perform bitwise OR on the values of Vx and Vy and
            // then store the result in Vx
            (0x8, _, _, 0x1) => todo!("OR Vx, Vy"),
            // AND Vx. Vy
            // Set Vx = Vx AND Vy
            // Perform bitwise AND on the values of Vx and Vy and
            // then store the result in Vx
            (0x8, _, _, 0x2) => todo!("AND Vx, Vy"),
            // XOR Vx, Vy
            // Set Vx = VX XOR Vy
            // Performs bitwise exclusive OR on the values of Vx and Vy and
            // then store the result in Vx
            (0x8, _, _, 0x3) => todo!("XOR Vx, Vy"),
            // ADD Vx, Vy
            // Set Vx = Vx + Vy, set VF = carry
            // The values of Vx and Vy are added together
            // If the result is greater than 8 bits, VF is set to 1, otherwise 0
            // ONly the lowest 8 bits of the result are kept and stored in Vx
            (0x8, _, _, 0x4) => todo!("ADD Vx, Vy"),
            // SUB Vx, Vy
            // Set Vx = Vx - Vy, set VF = not borrow
            // If Vx > Vy, then VF is set to 1, othwerwise 0
            // Then Vy is szbtracted from Vx and the result stored in Vx
            (0x8, _, _, 0x5) => todo!("SUB Vx, Vy"),
            // SHR Vx {, Vy}
            // Set Vx = Vx SHR 1
            // If the least-significant bit of Vx is 1,
            // then VF is set to 1, otherwise 0, then Vx is devided by 2
            (0x8, _, _, 0x6) => todo!("SHR Vx {}", "{, Vy}"),
            // SUBN Vx, Vy
            // Set Vx = Vy - Vx, set VF = NOT borrow
            // If Vy > Vy, then VF is 1, otherwise 0
            // Then Vx is subtracted from Vy, and the result stored in Vx
            (0x8, _, _, 0x7) => todo!("SUBN Vx, Vy"),
            // SHL Vx  {, Vy}
            // Set Vx 0 Vx SHL 1
            // If the most-significant bit of Vx is 1,
            // then VF is set to 1, otherwise 0, then Vx is devided by 2
            (0x8, _, _, 0xE) => todo!("SHL Vx {}", "{, Vy}"),

            // SNE Vx, Vy
            // Skip next instruction if Vx != Vy
            // Compare register Vx to vY and
            // if they are not equal increment pc by 2
            (0x9, _ , _, 0x0) => todo!("SNE Vx, Vy"),

            // LD I, addr
            // Set I = nnn
            // The value of register I is set to nnn
            (0xA, _, _, _) => self.i = nnn,

            // JV V0, addr
            // Jump to location nnn + V0
            // pc is set to nnn plus the value of V0
            (0xB, _, _, _) => todo!("JV V0, addr"),

            // RND Vx, byte
            // Set Vx = random byte AND kk
            // generate random number betweeon 0 and 255,
            // which is then ANDed with the value of kk
            // the result us stored in Vx
            // (See Instruction 8xy2 for AND)
            (0xC, _, _, _) => todo!("RND Vx, byte"),

            // DRW Vx, Vy, nibble
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            // Taken directly from http://devernay.free.fr/hacks/chip8/C8TECH10.HTM: 
            // The interpreter reads n bytes from memory, starting at the address stored in I.
            // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
            // Sprites are XORed onto the existing screen.
            // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
            // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen.
            // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
            (0xD, _, _, _) => {
                let collision = self.display.draw(vx as usize, vy as usize,
                    &self.memory[self.i as usize .. (self.i + n as u16) as  usize]);
                self.v[0xF] = if collision { 1 } else { 0 };
            },

            // SKP Vy
            // Skip next instrucion if the key with the value of Vx is pressed
            // Check the keyboard, and if the key corresponding to the value of
            // Vx is currently in the down position, pc is increased by 2
            (0xE, _, 0x9, 0xE) => todo!("SKP Vy"),

            // SKNP Vy
            // Skip next instrucion if the key with the value of Vx is not pressed
            // Check the keyboard, and if the key corresponding to the value of
            // Vx is currently in the up position, pc is increased by 2
            (0xE, _, 0xA, 0x1) => todo!("SKNP Vy"),

            // LD Vx, DT
            // Set Vx = delay timer value
            // The value of DT is placed into Vx
            (0xF, _, 0x0, 0x7) => todo!("LD Vx, DT"),

            // LD Vx, K
            // Wait for a key press, store the value of the key in Vx
            // All execution stops until a key is pressed, them the value of the key is stored in Vx
            (0xF, _, 0x0, 0xA) => todo!("LD Vx, K"),

            // LD DT, Vx
            // Set delay timer = Vx
            // DT os set equal to the value of Vx
            (0xF, _, 0x1, 0x5) => todo!("LD DT, Vx"),

            // LD ST, Vx
            // Set sound timer = Vx
            // ST is set equal to the value of Vx
            (0xF, _, 0x1, 0x8) => todo!("LD ST, Vx"),

            // ADD I, Vx
            // Set I = I + Vx
            // The values of I and Vx are added
            // the result is stored in I
            (0xF, _, 0x1, 0xF) => todo!("ADD I, Vx"),

            // LD F, Vx
            // Set I = location of sprite for digit Vx
            // The value of I is set to the location for the hexadecimal sprite
            // corresponding to the value of Vx.
            (0xF, _, 0x2, 0x9) => todo!("LD F, Vx"),

            // LD B, Vx
            // Store BCD representation of Vx in memory locations I, I+1 and I+2
            // Take the decimal value of Vx
            // and places a houndred digit in memory at location in i
            // and tens digit at location I+1
            // and ones digit at location I+2
            (0xF, _, 0x3, 0x3) => todo!("LD B, Vx"),

            // LD [I], Vx
            // Store registers V0 through Vx from memory starting at location I
            // Copy the values of register V0 through Vx into memory,
            // starting at address in I
            (0xF, _, 0x5, 0x5) => todo!("LD [I], Vx"),

            // LD Vx, [I]
            // Read registers V0 through Vx from memory starting at location I
            // Read the values from memory starting at location I
            // into registers V0 through Vx
            (0xF, _, 0x6, 0x5) => todo!("LD Vx, [I]"),

            // There is the possibility
            // to add further instructions for the Super Chip-48
            // It is "just" 10 more opcodes

            // Ignore all other codes...
            (_, _, _, _) => ()
        }
    }
}

fn read_word(memory: [u8; 4096], index: u16) -> u16 {
    (memory[index as usize] as u16) << 8 | (memory[(index + 1) as usize] as u16)
}

struct Window {}

impl Window {
    fn new() -> Self {
        Self {}
    }

    fn draw(&self, frame: &mut [u8], display: Display) {
        let mut d = display;
        for(i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as usize;
            let y = (i / WIDTH as usize) as usize;

            let rgba = if d.get_pixel(x, y) {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixel.copy_from_slice(&rgba);
        }
    }
}

fn render (mut chip: Cpu) {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new((WIDTH * 15) as f64, (HEIGHT * 15) as f64);
        WindowBuilder::new()
            .with_title("Chip-8 Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };
    let win = Window::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // One of the most important functions ...
        // letting the cpu execute an cycle
        chip.execute_cycle();

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                win.draw(pixels.frame_mut(), chip.display);
                if let Err(_) = pixels.render() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            // Redraw the window ... again
            Event::MainEventsCleared => window.request_redraw(),
            _ => {}
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            window.request_redraw();
        }
    });
    }

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?} arguments: {:?}.", args.len() - 1, &args[1..]);

    // Declare the chip
    let mut chip = Cpu::new();
    // Reset the chip
    chip.reset();
    // Load an ROM
    chip.load_rom("rom/bin/test_opcode.ch8".to_string()).unwrap();
    // render the shit
    render(chip);
}
