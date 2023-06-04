use crate::cpu::cpu::CPU;
use bitflags::bitflags;

use std::collections::HashMap;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u16 {
        const CLEAR_FLAG = 0;
        const Z = (1 << 0); //zero
        const C = (1 << 1); //cary
        const N = (1 << 2); //negative
        const O = (1 << 3); //overflow
        const P = (1 << 4); //
        //const ALLFLAGS = Self::Z.bits() | Self::C.bits() | Self::N.bits() | Self::O.bits() | Self::P.bits();
    }
}


//impl Flags {
    pub fn gen_flag(enc: &Encoding, cpu: &CPU) -> u16 {
        let mut temp_flag = Flags::CLEAR_FLAG;
        if cpu.state.reg[enc.rd as usize] == 0 { temp_flag |= Flags::Z; }
        //TODO: rest of flags implementations here\   
        
        /*example debug:*/ //println!("{:?}", temp_flag);
        temp_flag.bits()
    }
//}


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
    AND = 0x13,
    ORR = 0x14,
    XOR = 0x15,
    ANI = 0x16,
    ORI = 0x17,
    XOI = 0x18,
    SHL = 0x19,
    SHR = 0x20,
    SLI = 0x23,
    SRI = 0x24,
    DIV = 0x1D,
    MUL = 0x1C,
    MOD = 0x2C,

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
        m.insert(Opcode::AND as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] & cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("and r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ORR as u8, Operation {
        execute: |enc, cpu| {
            cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] | cpu.state.reg[enc.rs2 as usize];
            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::XOR as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] ^ cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("xor r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ANI as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] + enc.imm;
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ani r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::ORI as u8, Operation {
        execute: |enc, cpu| {
            cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] | enc.imm;
            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ori r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::XOI as u8, Operation {
        execute: |enc, cpu| {
            cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] ^ enc.imm;
            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("xoi r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SHL as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] << cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("shl r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SHR as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] >> cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("shr r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SLI as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] << enc.imm;
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("sli r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SRI as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] >> enc.imm;
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("sri r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::DIV as u8, Operation {
           execute: |enc, cpu| {
               cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] / cpu.state.reg[enc.rs2 as usize];
               cpu.state.pc = cpu.state.pc+1;
           },
           repr: |enc| format!("div r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::MUL as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] * cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("mul r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::MOD as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize] % cpu.state.reg[enc.rs2 as usize];
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("mod r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
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
