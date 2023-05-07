use std::io;
use std::io::prelude::*;
use std::fs::File;

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
                    let xi = (x + 1) % WIDTH;
                    let yi = (y + 1) % HEIGHT;
                    let old_value = self.get_pixel(xi, yi);
                    if old_value {
                        collision = true;
                    }
                    self.set_pixel(xi, yi, (new_value == 1) ^ old_value);
                }
            }
        }
        return  collision;
    }
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
        self.pc = 0x200;
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.dt = 0;
        self.display.cls();
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
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        // break up int nibble (1/2 byte)
        let op_1 = (opcode & 0xF000) >> 12;
        let op_2 = (opcode & 0x0F00) >> 8;
        let op_3 = (opcode & 0x00F0) >> 4;
        let op_4 = opcode & 0x000F;

        // increment the counter
        self.pc += 2;

        // println!("{:X}, {:X}, {:X}, {:X}", op_1, op_2, op_3, op_4);

        match (op_1, op_2, op_3, op_4) {
            // CLS
            (0, 0, 0xE, 0) => self.display.cls(),
            // RET
            (0, 0, 0xE, 0xE) => todo!("'RET' not implemented yet!"),
            // JP addr
            (0x1, _, _, _) => self.pc = nnn,
            // LD Vx, byte
            (0x6, _, _, _) => self.v[x] = kk,
            // ADD Vx, byte
            (0x7, _, _, _) => self.v[x] = self.v[x] + kk,
            // LD I, addr
            (0xA, _, _, _) => self.i = nnn,
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let collision = self.display.draw(vx as usize, vy as usize,
                    &self.memory[self.i as usize .. (self.i + n as u16) as  usize]);
                self.v[0xF] = if collision { 1 } else { 0 };
            },
            // Unknown Opcode
            (_, _, _, _) => (),
        }
    }
}

fn read_word(memory: [u8; 4096], index: u16) -> u16 {
    (memory[index as usize] as u16) << 8 | (memory[(index + 1) as usize] as u16)
}

fn display(mut display: Display) {
    // for i in 0..display.memory.len() {
    //     print!("{} ", i);
    // }

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if display.get_pixel(x, y) {
                print!("*")
            } else {
                print!(" ")
            }
        }
        println!("")
    }
}

fn main() -> io::Result<()> {
    let mut chip = Cpu::new();
    chip.reset();
    chip.load_rom("rom/IBM".to_string()).unwrap();
    loop {
        chip.execute_cycle();
    }
}
