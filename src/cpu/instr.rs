use crate::cpu::cpu::CPU;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Encoding {
    opcode : u8,
    rd : u8,
    rs1 : u8,
    rs2 : u8,
    imm : u16,
}

fn extract(field: u32, start: usize, len: usize) -> u32 {
   (field>>start)&((1<<len)-1)
}

impl Encoding {
    pub fn from_raw(instr: u32) -> Self {
        Self {opcode: extract(instr, 0,  6) as u8,
              rd:     extract(instr, 7,  3) as u8,
              rs1:    extract(instr, 10, 3) as u8,
              rs2:    extract(instr, 13, 3) as u8,
              imm:    extract(instr, 16, 16) as u16
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
enum Opcode {
    NOP = 0x0,
    LDD = 0x2,
    ADD = 0x7,
}

struct Operation {
    execute: fn(&Encoding, &mut CPU),
    repr: fn(&Encoding) -> String,
}


lazy_static! {
    static ref OP_MAP : HashMap<u8, Operation> = {  
        let mut m = HashMap::new();

        m.insert(Opcode::NOP as u8, Operation {
            execute: |_enc, cpu| {
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |_enc| String::from("nop"),
        }); 
        m.insert(Opcode::LDD as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.read(enc.imm, true);
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ldd r{0}, {1}", enc.rd, enc.imm),
        });
        m.insert(Opcode::ADD as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] + cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("add r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });

        m
    };
}

pub fn execute(enc: &Encoding, cpu: &mut CPU) {
    let op = OP_MAP.get(&enc.opcode)
        .unwrap_or_else(|| {
            println!("unknown operation {:?}", enc);
            OP_MAP.get(&(Opcode::NOP as u8)).unwrap()
    });
    
    (op.execute)(enc, cpu);
}
