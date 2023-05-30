use crate::cpu::cpu::State;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Encoding {
    opcode : u8,
    rd : u8,
    rs1 : u8,
    rs2 : u8,
    imm : u16,
}

impl Encoding {
    fn from_bytes(bytes: [u8; 4]) -> Self {
        Self {opcode:0, rd:1, rs1:0, rs2:0, imm:1}
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
enum Opcode {
    NOP = 0,
    ADD = 7,
    AND = 13,
    ORR = 14,
    XOR = 15,
    ANI = 16,
    ORI = 17,
    XOI = 18,
    SHL = 19,
    SHR = 20,
    
}

struct Operation {
    execute: fn(&Encoding, &mut State),
    repr: fn(&Encoding) -> String,
}


lazy_static! {
    static ref OP_MAP : HashMap<u8, Operation> = {  
        let mut m = HashMap::new();

        m.insert(Opcode::NOP as u8, Operation {
            execute: |_enc, state| {
                state.pc = state.pc+1;
            },
            repr: |_enc| String::from("nop"),
        }); 
        m.insert(Opcode::ADD as u8, Operation {
            execute: |enc, state| {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] + state.reg[enc.rs2 as usize];
                state.pc = state.pc+1;
            },
            repr: |enc| format!("add r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::AND as u8, Operation {
            execute: |enc, state | {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] & state.reg[enc.rs2 as usize];
                state.pc = state.pc+1;
            },
            repr: |enc| format!("and r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ORR as u8, Operation {
        execute: |enc, state | {
            state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] | state.reg[enc.rs2 as usize];
            state.pc = state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
       m.insert(Opcode::XOR as u8, Operation {
            execute: |enc, state | {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] ^ state.reg[enc.rs2 as usize];
                state.pc = state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
       m.insert(Opcode::ANI as u8, Operation {
            execute: |enc, state| {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] + state.reg[enc.imm as usize];
                state.pc = state.pc+1;
            },
            repr: |enc| format!("add r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
       m.insert(Opcode::ORI as u8, Operation {
        execute: |enc, state | {
            state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] | state.reg[enc.imm as usize];
            state.pc = state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
       m.insert(Opcode::XOI as u8, Operation {
        execute: |enc, state | {
            state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] ^ state.reg[enc.imm as usize];
            state.pc = state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::XOI as u8, Operation {
            execute: |enc, state | {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] ^ state.reg[enc.imm as usize];
                state.pc = state.pc+1; 
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SHL as u8, Operation {
            execute: |enc, state | {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] << state.reg[enc.rs2 as usize];
                state.pc = state.pc+1; 
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SHR as u8, Operation {
            execute: |enc, state | {
                state.reg[enc.rd as usize] = state.reg[enc.rs1 as usize] >> state.reg[enc.rs2 as usize];
                state.pc = state.pc+1; 
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
         
  

        m
    };
}

pub fn execute(enc: &Encoding, cpu: &mut State) {
    let op = OP_MAP.get(&enc.opcode)
        .unwrap_or_else(|| {
            println!("unknown operation {:?}", enc);
            OP_MAP.get(&(Opcode::NOP as u8)).unwrap()
    });
    
    (op.execute)(enc, cpu);
}
