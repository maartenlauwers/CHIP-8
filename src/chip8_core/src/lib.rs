use rand::RngExt;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200; // 512 bytes

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
    halted: bool,
    halted_vx: u8
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            halted: false,
            halted_vx: 0
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
        if self.halted {
            self.v_reg[self.halted_vx as usize] = idx as u8;
            self.halted = false;
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Caused by instruction Fx0A.
        if self.halted { return }

        // Fetch
        let op = self.fetch();
        // Decode & Execute
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        // NOTE: The | performs a bitwise OR, so it's commutative.
        let op  = (higher_byte << 8) | lower_byte;
        // NOTE: Technically, this can overflow. But as this would be
        // a fault in either the program or our emulator, we want it
        // to panic.
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;
        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => {
                println!("00E0");
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]
            },
            (0, 0, 0xE, 0xE) => {
                println!("00EE");
                // 00EE: Return from subroutine.
                self.pc = self.pop()
            }
            (1, _, _, _) => {
                println!("1nnn");
                // 1nnn: Jump to location nnn.
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (2, _, _, _) => {
                println!("2nnn");
                // 2nnn: Call subroutine at nnn.
                self.push(self.pc);
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            (3, _, _, _) => {
                println!("3xkk");
                // 3xkk: Skip next instruction if Vx = kk.
                let x= self.v_reg[digit2 as usize];
                let kk = (op & 0xFF) as u8;
                if x == kk {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                println!("4xkk");
                // 4xkk: Skip next instruction if Vx != kk.
                let x = self.v_reg[digit2 as usize];
                let kk = (op & 0xFF) as u8;
                if x != kk {
                    self.pc += 2;
                }
            }
            (5, _, _ , 0) => {
                println!("5xy0");
                // 5xy0: Skip next instruction if Vx = Vy.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                println!("x={}, y={}", x, y);
                if x == y {
                    self.pc += 2
                }
            }
            (6, _, _, _) => {
                println!("6xkk");
                // 6xkk: Set Vx = kk
                let kk = (op & 0xFF) as u8;
                println!("kk={}", kk);
                self.v_reg[digit2 as usize] = kk;
            }
            (7, _, _, _) => {
                println!("7xkk");
                // 7xkk: Adds the value kk to the value Vx, then stores the result in Vx.
                // NOTE: The addition should wrap around! Apparently this is undefined but correct
                // behaviour in C.
                let x = self.v_reg[digit2 as usize] as u8;
                let kk = (op & 0xFF) as u8;
                self.v_reg[digit2 as usize] = x.wrapping_add(kk);
            }
            (8, _, _, 0) => {
                println!("8xy0");
                // 8xy0: Stores the value of register Vy in register Vx
                let y = self.v_reg[digit3 as usize];
                self.v_reg[digit2 as usize] = y;
            }
            (8, _, _, 1) => {
                println!("8xy1");
                // 8xy1: Set Vx = Vx OR Vy
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                self.v_reg[digit2 as usize] = x | y;
            }
            (8, _, _, 2) => {
                println!("8xy2");
                // 8xy2: Set Vx = Vx AND Vy.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                self.v_reg[digit2 as usize] = x & y;
            }
            (8, _, _, 3) => {
                println!("8xy3");
                // 8xy3: Set Vx = Vx XOR Vy.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                self.v_reg[digit2 as usize] = x ^ y;
            }
            (8, _, _, 4) => {
                println!("8xy4");
                // 8xy4: Set Vx = Vx + Vy, set VF = carry.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                let (result, carry) = x.overflowing_add(y);
                self.v_reg[digit2 as usize] = result;
                self.v_reg[0xF] = if carry { 1 } else { 0 }
            }
            (8, _, _, 5) => {
                println!("8xy5");
                // 8xy5: Set Vx = Vx - Vy, set VF = NOT borrow.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                self.v_reg[digit2 as usize] = x.wrapping_sub(y);
                self.v_reg[0xF] = if x > y { 1 } else { 0 }
            }
            (8, _, _, 6) => {
                println!("8xy6");
                // 8xy6: Set Vx = Vx SHR 1.
                // NOTE: Original interpreter performs Vx = Vy >> 1.
                // We do Vx = Vx >> 1. Some ROMs may require the former, others the
                // latter. To see how this behaves when testing.
                let x = self.v_reg[digit2 as usize];
                let lsb = x & 1;
                self.v_reg[0xF] = if lsb == 1 { 1 } else { 0 };
                self.v_reg[digit2 as usize] = x / 2;
            }
            (8, _, _, 7) => {
                println!("8xy7");
                // 8xy7: Set Vx = Vy - Vx, set VF = NOT borrow.
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                self.v_reg[0xF] = if y > x { 1 } else { 0 };
                self.v_reg[digit2 as usize] = y.wrapping_sub(x);
            }
            (8, _, _, 0xE) => {
                println!("8xyE");
                // 8xyE: Set Vx = Vx SHL 1.
                let x = self.v_reg[digit2 as usize];
                let msb = x >> 0x7;
                self.v_reg[0xF] = msb;
                self.v_reg[digit2 as usize] = x.wrapping_mul(2);
            }
            (9, _, _, 0) => {
                println!("9xy0");
                // 9xy0: Skip next instruction if Vx != Vy
                let x = self.v_reg[digit2 as usize];
                let y = self.v_reg[digit3 as usize];
                if x != y {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                println!("Annn");
                // Annn: Set I = nnn.
                self.i_reg = op & 0x0FFF
            }
            (0xB, _, _, _) => {
                println!("Bnnn");
                // Bnnn: Jump to location nnn + V0
                let v0 = self.v_reg[0];
                let nnn = op & 0x0FFF;
                self.pc = (nnn + v0 as u16) & 0x0FFF;
            }
            (0xC, _, _, _) => {
                println!("Cxkk");
                // Cxkk: Set Vx = random byte and kk.
                let byte: u8 = rand::rng().random();
                let kk =(op & 0xFF) as u8;
                self.v_reg[digit2 as usize] = byte & kk;
            },
            (0xD, _, _, _) => {
                println!("Dxyn");
                // Dxyn: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let start = self.i_reg as usize;
                let end = start + digit4 as usize;
                let sprite = &self.ram[start..end];
                let x = self.v_reg[digit2 as usize] as u64;
                let y = self.v_reg[digit3 as usize] as u64;

                // Set VF to 0
                self.v_reg[0xF] = 0;

                let mut row = y % SCREEN_HEIGHT as u64;
                let col = x % SCREEN_WIDTH as u64;

                println!("x={},y={}",x,y);

                for byte in sprite {
                    for x in 0..8 {
                        // Calculate updated column, with wrap around
                        let mut new_col = col + x;
                        let screen_width = SCREEN_WIDTH as u64;
                        if new_col >= screen_width { new_col = new_col % screen_width}
                        // if new_col >= screen_width { continue };

                        let row_offset = row * SCREEN_WIDTH as u64;
                        let screen_index = (row_offset + new_col as u64) as usize;

                        let current = self.screen[screen_index];
                        let bit = 0b1000_0000 >> x;
                        let has_pixel = byte & bit != 0;
                        self.screen[screen_index] ^= has_pixel;
                        if current && has_pixel {
                            self.v_reg[0xF] = 1;
                        }
                    }
                    // Set offset to next row.
                    row += 1;
                    if row >= SCREEN_HEIGHT as u64 {
                        row = 0;
                    }
                }
            }
            (0xE, _, 9, 0xE) => {
                println!("Ex9E");
                // Ex9E: Skip next instruction if key with the value of Vx is pressed
                let x = self.v_reg[digit2 as usize];
                if self.keys[x as usize] == true {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                println!("ExA1");
                // ExA1: Skip next instruction if key with the value of Vx is not pressed.
                let x = self.v_reg[digit2 as usize];
                if self.keys[x as usize] == false {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                println!("Fx07");
                // Fx07: Set Vx = delay timer value.
                self.v_reg[digit2 as usize] = self.dt;
            }
            (0xF, _, 0, 0xA) => {
                println!("Fx0A");
                // Fx0A: Wait for a key press, store the value of the key in Vx.
                self.halted = true;
                self.halted_vx = digit2 as u8;
                return
            }
            (0xF, _, 1, 5) => {
                println!("Fx15");
                // Fx15: Set delay timer = Vx
                self.dt = self.v_reg[digit2 as usize];
            }
            (0xF, _, 1, 8) => {
                println!("Fx18");
                // Fx18: Set sound timer = Vx.
                self.st = self.v_reg[digit2 as usize];
            }
            (0xF, _, 1, 0xE) => {
                println!("Fx1E");
                // Fx1E: Set I = I + Vx.
                self.i_reg = self.i_reg + self.v_reg[digit2 as usize] as u16;
            }
            (0xF, _, 2, 9) => {
                println!("Fx29");
                // Fx29: Set I = location of sprite for digit Vx.
                self.i_reg = (self.v_reg[digit2 as usize] as u16) * 5;
            }
            (0xF, _, 3, 3) => {
                println!("Fx33");
                // Fx33: Store BCD representation of Vx in memory locations I, I+1, and I+2.
                let x  = self.v_reg[digit2 as usize];
                let i = self.i_reg as usize;
                self.ram[i] = x / 100;
                self.ram[i + 1] = (x % 100) / 10;
                self.ram[i + 2] = x % 10
            }
            (0xF, _, 5, 5) => {
                println!("Fx55");
                // Fx55: Store registers V0 through Vx in memory starting at location I
                let i = self.i_reg as usize;
                let x = digit2;
                for index in 0..=x {
                    self.ram[i + index as usize] = self.v_reg[index as usize];
                }
            }
            (0xF, _, 6, 5) => {
                println!("Fx65");
                // Fx65: Read registers V0 through Vx from memory starting at location I
                let i = self.i_reg as usize;
                let x = digit2;
                for index in 0..=x {
                    self.v_reg[index as usize] = self.ram[i + index as usize];
                }
                self.i_reg = i as u16 + x as u16 + 1;
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // TODO: Bug in guide? Spec says it should beep
                // while the sound timer is non-zero.
                // BEEP
            }
            self.st -=1 ;
        }
    }
}