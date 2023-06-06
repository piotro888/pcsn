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
        const P = (1 << 4); //parity
    }
}

pub fn convert_to_bitflags(mut field: u16) -> Flags {
    let mut temp_flag: Flags = Flags::CLEAR_FLAG;
    if field & 0b1 == 1 { temp_flag |= Flags::Z; }
    field = field >> 1;

    if field & 0b1 == 1 { temp_flag |= Flags::C; }
    field = field >> 1;

    if field & 0b1 == 1 { temp_flag |= Flags::N; }
    field = field >> 1;

    if field & 0b1 == 1 { temp_flag |= Flags::O; }
    field = field >> 1;

    if field & 0b1 == 1 { temp_flag |= Flags::P; }
    field = field >> 1;

    return temp_flag;
}

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
enum Opcode { /* "//" <- means ready */
    NOP = 0x0,//
    MOV = 0x1,//
    LDD = 0x2,
    ADD = 0x7,//
    ADI = 0x8,//
    SUB = 0xA,//
    CMP = 0xC,//
    CMI = 0xD,//
    JMP = 0xE,
    JAL = 0xF,//
    AND = 0x13,//
    ORR = 0x14,//
    XOR = 0x15,//
    ANI = 0x16,//
    ORI = 0x17,//
    XOI = 0x18,//
    SHL = 0x19,//
    SHR = 0x1A,//
    SLI = 0x23,//
    SRI = 0x24,//
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
        m.insert(Opcode::MOV as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.reg[enc.rs1 as usize];
            },
            repr: |enc| format!("mov r{}, r{}", enc.rd, enc.rs1),
        });
        m.insert(Opcode::LDD as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.read(enc.imm, true);
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ldd r{0}, {1}", enc.rd, enc.imm),
        });
        m.insert(Opcode::ADD as u8, Operation { //changed implementation, because then gen_flag function would loose bits and wouldn't work.
            execute: |enc, cpu|{
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 + cpu.state.reg[enc.rs2 as usize] as u32; //this gives us acces to 17th bit
                cpu.state.reg[enc.rd as usize] = _out as u16; // but the same value as before goes back to register
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out); 
                //happy genflag function here^
                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("add r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ADI as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 + enc.imm as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("adi r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SUB as u8, Operation {
            execute: |enc, cpu| { 
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 - cpu.state.reg[enc.rs2 as usize] as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs1 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("sub r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::AND as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 & cpu.state.reg[enc.rs2 as usize] as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("and r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ORR as u8, Operation {
        execute: |enc, cpu| {
            let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 | cpu.state.reg[enc.rs2 as usize] as u32;
            cpu.state.reg[enc.rd as usize] = _out as u16;
            cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("or r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::XOR as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 ^ cpu.state.reg[enc.rs2 as usize] as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("xor r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::ANI as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32+ enc.imm as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs1 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ani r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::ORI as u8, Operation {
        execute: |enc, cpu| {
            let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 | enc.imm as u32;
            cpu.state.reg[enc.rd as usize] = _out as u16;
            cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("ori r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::XOI as u8, Operation {
        execute: |enc, cpu| {
            let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 ^ enc.imm as u32;
            cpu.state.reg[enc.rd as usize] = _out as u16;
            cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

            cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("xoi r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SHL as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = ((cpu.state.reg[enc.rs1 as usize] as u32) << (cpu.state.reg[enc.rs2 as usize] as u32)); //i have no fucking idea why this doesnt want to work,it's litteraly the same as other functions mijv6fi2j535f5r5gejihnegfboegoje9ef9nhef9e
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("shl r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SHR as u8, Operation {
            execute: |enc, cpu| {
                let _out:u32 = cpu.state.reg[enc.rs1 as usize] as u32 >> cpu.state.reg[enc.rs2 as usize] as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("shr r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
        });
        m.insert(Opcode::SLI as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = ((cpu.state.reg[enc.rs1 as usize] as u32) << (enc.imm as u32));
                cpu.state.reg[enc.rd as usize] = _out as u16; // again wtf????
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("sli r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SRI as u8, Operation {
            execute: |enc, cpu| {
                let _out:u32 = cpu.state.reg[enc.rs1 as usize] as u32 >> enc.imm as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs1 as usize] as u32, _out);
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
        m.insert(Opcode::CMP as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 - cpu.state.reg[enc.rs2 as usize] as u32;
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs1 as usize] as u32, _out)
            },
            repr: |enc| format!("cmp r{}, r{}", enc.rs1, enc.rs2),
        });
        m.insert(Opcode::CMI as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 - enc.imm as u32;
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);
            },
            repr: |enc| format!("cmp r{}, {}", enc.rs1, enc.imm),
        });
        m.insert(Opcode::JAL as u8, Operation {
            execute: |enc, cpu| {
                cpu.state.reg[enc.rd as usize] = cpu.state.pc;
                cpu.state.pc = enc.imm; 
            },
            repr: |enc| format!("jal {}", enc.imm), //probably should be better
        });
        m.insert(Opcode::JMP as u8, Operation {
            execute: |enc, cpu| {
                let jmp_code: u8 = ((enc.rs1 << 3) as u8) +  enc.rd as u8;
                let cpu_flags: Flags = convert_to_bitflags(cpu.state.flags);

                match jmp_code {
                    0x0 => cpu.state.pc = enc.imm,
                    0x1 => if cpu_flags & Flags::C == Flags::C { cpu.state.pc = enc.imm},
                    0x2 => if cpu_flags & Flags::Z == Flags::Z { cpu.state.pc = enc.imm},
                    0x3 => if cpu_flags & Flags::N == Flags::N { cpu.state.pc = enc.imm},
                    0x4 => if cpu_flags & (Flags::N | Flags::Z) != (Flags::N | Flags::Z) { cpu.state.pc = enc.imm },
                    0x5 => if cpu_flags & (Flags::N | Flags::Z) == (Flags::N | Flags::Z) { cpu.state.pc = enc.imm },
                    0x6 => if cpu_flags & Flags::N != Flags::N { cpu.state.pc = enc.imm},
                    0x7 => if cpu_flags & Flags::Z != Flags::Z { cpu.state.pc = enc.imm},
                    0x8 => if cpu_flags & Flags::O == Flags::O { cpu.state.pc = enc.imm},
                    0x9 => if cpu_flags & Flags::P == Flags::P { cpu.state.pc = enc.imm},
                    0xA => if cpu_flags & (Flags::C | Flags::Z) != (Flags::C | Flags::Z) { cpu.state.pc = enc.imm },
                    0xB => if cpu_flags & Flags::C != Flags::C { cpu.state.pc = enc.imm},
                    0xC => if cpu_flags & (Flags::C | Flags::Z) == (Flags::C | Flags::Z) { cpu.state.pc = enc.imm },
                    _ => println!("jmp jump_code error! invalid instruction."),
                }




            },
            repr: |enc| { format!("jumped to {}", enc.imm) }, //looks ugly... TODO: do it better
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

fn gen_flag(is_subtract: bool, var_1: u32, var_2: u32, mut var_out: u32) -> u16 {
    let mut temp_flag = Flags::CLEAR_FLAG;
    if extract(var_out, 15, 1) == 1 { temp_flag |= Flags::N; }
    
    if extract(var_out, 16, 1) == 1 { temp_flag |= Flags::C; }

    if is_subtract {
        if (extract(var_1, 15, 1) ^ extract(var_2, 15, 1) ^ 1) & (extract(var_1, 15, 1) ^ extract(var_out, 15, 1)) == 1 { temp_flag |= Flags::O; }
    }
    else {
        if (extract(var_1, 15, 1) ^ extract(var_2, 15, 1) ^ 0) & (extract(var_1, 15, 1) ^ extract(var_out, 15, 1)) == 1 { temp_flag |= Flags::O; }
    }

    if var_out as u16 == 0 { temp_flag |= Flags::Z; }

    //bit magic
    var_out ^= var_out >> 8;
    var_out ^= var_out >> 4;
    var_out ^= var_out >> 2;
    var_out ^= var_out >> 1;
    if extract(var_out, 0, 1) == 0 { temp_flag |= Flags::P; }

    println!("{:?}", temp_flag);
    temp_flag.bits()
}
