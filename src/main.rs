// const zero : [u8; 5] = [0xF0, 0x90, 0x90, 0x90, 0xF0];
// const one  : [u8; 5] = [0x20, 0x60, 0x20, 0x20, 0x70];
// const two  : [u8; 5] = [0xF0, 0x10, 0xF0, 0x80, 0xF0];
// const tree : [u8; 5] = [0xF0, 0x10, 0xF0, 0x10, 0xF0];
// const four : [u8; 5] = [0x90, 0x90, 0xF0, 0x10, 0x10];
// const five : [u8; 5] = [0xF0, 0x80, 0xF0, 0x10, 0xF0];
// const six  : [u8; 5] = [0xF0, 0x80, 0xF0, 0x90, 0xF0];
// const seven: [u8; 5] = [0xF0, 0x10, 0x20, 0x40, 0x40];
// const eight: [u8; 5] = [0xF0, 0x90, 0xF0, 0x90, 0xF0];
// const nine : [u8; 5] = [0xF0, 0x90, 0xF0, 0x10, 0xF0];
// const a    : [u8; 5] = [0xF0, 0x90, 0xF0, 0x90, 0x90];
// const b    : [u8; 5] = [0xE0, 0x90, 0xE0, 0x90, 0xE0];
// const c    : [u8; 5] = [0xF0, 0x80, 0x80, 0x80, 0xF0];
// const d    : [u8; 5] = [0xE0, 0x90, 0x90, 0x90, 0xE0];
// const e    : [u8; 5] = [0xF0, 0x80, 0xF0, 0x80, 0xF0];
// const f    : [u8; 5] = [0xF0, 0x80, 0xF0, 0x80, 0x80];

use std::io;
use std::io::prelude::*;
use std::fs::File;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

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
    pub fn draw() {
        // ...
    }
}

pub struct Cpu {
    // index register
    pub l: u16,
    // program counter
    pub pc: u16,
    // memory
    pub memory: [u8; 4096],
    // register
    pub v: [u8; 16],
    // peropherals
    // pub keypad: Kaypad,
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
            l: 0,
            pc: 0,
            memory: [0; 4096],
            v: [0; 16],
            display: Display::new(),
            stack: [0; 16],
            sp: 0,
            dt: 0,
        }
    }

    pub fn load_rom(&mut self, file_path: String) -> io::Result<u16> {
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer)?;
        for i in 0..buffer.len() {
            self.memory[i] = buffer[i];
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
        // let x = ((opcode & 0x0F00) >> 8) as usize;
        // let y = ((opcode & 0x00F0) >> 4) as usize;
        // let vx = self.v[x];
        // let vy = self.v[y];
        // let nnn = opcode & 0x0FFF;
        // let kk = (opcode & 0x00FF) as u8;
        // let n = (opcode & 0x000F) as u8;

        // break up int nibble (1/2 byte)
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        // increment the counter
        self.pc += 2;

        // println!("{}, {}, {}, {}", op_1, op_2, op_3, op_4);

        match (op_1, op_2, op_3, op_4) {
            // CLS
            (0, 0, 0xE, 0) => self.display.cls(),
            // RET
            (0, 0, 0xE, 0xE) => todo!("'RET' not implemented yet!"),
            // JMP
            (0x1, _, _, _) => todo!("'JMP' not implemented yet!"),
            // LD Vx, byte
            (0x6, _, _, _) => todo!("'LD Vx, byte' not implemented yet!"),
            // ADD Vx, byte
            (0x7, _, _, _) => todo!("'ADD Vx, byte' not implemted yet!"),
            // LD I, addr
            (0xA, _, _, _) => todo!("'LD I, addr' not implemented yet!"),
            // DRW Vx, VY, nibble
            (0xD, _, _, _) => todo!("'DRW Vx, VY, nibble' not impelmented yet!"),
            // Unknown Opcode
            (_, _, _, _) => todo!("Unknown opcode"),
        }
    }
}

fn read_word(memory: [u8; 4096], index: u16) -> u16 {
    (memory[index as usize] as u16) << 8 | (memory[(index + 1) as usize] as u16)
}

fn main() -> io::Result<()> {
    let mut chip = Cpu::new();
    let lenght = chip.load_rom("rom/IBM".to_string()).unwrap();
    for _ in 0..lenght {
        chip.execute_cycle ();
    }
    Ok(())

}
