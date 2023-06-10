use crate::cpu::cpu::CPU;
use bitflags::bitflags;

use std::collections::HashMap;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Flags: u16 {
        const Z = (1 << 0); //zero
        const C = (1 << 1); //cary
        const N = (1 << 2); //negative
        const O = (1 << 3); //overflow
        const P = (1 << 4); //parity
    }
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
enum Opcode {
    NOP = 0x0,
    MOV = 0x1,
    LDD = 0x2,
    ADD = 0x7,
    ADI = 0x8,
    SUB = 0xA,
    CAI = 0xB,
    CMP = 0xC,
    CMI = 0xD,
    JMP = 0xE,
    JAL = 0xF,
    AND = 0x13,
    ORR = 0x14,
    XOR = 0x15,
    ANI = 0x16,
    ORI = 0x17,
    XOI = 0x18,
    SHL = 0x19,
    SHR = 0x1A,
    SLI = 0x23,
    SRI = 0x24,
    SAR = 0x25,
    SAI = 0x26,
    SEX = 0x27,
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
                
                cpu.state.pc = cpu.state.pc+1;
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
        m.insert(Opcode::ADD as u8, Operation {
            execute: |enc, cpu|{
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 + cpu.state.reg[enc.rs2 as usize] as u32; // this gives us acces to 17th bit
                cpu.state.reg[enc.rd as usize] = _out as u16; // but the correct value goes back to register
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out); 
                
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
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);

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
            repr: |enc| format!("orr r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
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
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 & enc.imm as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

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
                let _out: u32 = ((cpu.state.reg[enc.rs1 as usize] as u32) << (cpu.state.reg[enc.rs2 as usize] as u32)); // necessary parentheses  
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
                let _out: u32 = ((cpu.state.reg[enc.rs1 as usize] as u32) << (enc.imm as u32));// necessary parentheses
                cpu.state.reg[enc.rd as usize] = _out as u16; 
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);

                cpu.state.pc = cpu.state.pc+1;
            },
            repr: |enc| format!("sli r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
        });
        m.insert(Opcode::SRI as u8, Operation {
            execute: |enc, cpu| {
                let _out:u32 = cpu.state.reg[enc.rs1 as usize] as u32 >> enc.imm as u32;
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);
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
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out)
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
            repr: |enc| format!("jal {} {}",enc.rd, enc.imm),
        });
        m.insert(Opcode::JMP as u8, Operation {
            execute: |enc, cpu| {
                let jmp_code: u8 = ((enc.rs1 << 3) as u8) +  enc.rd as u8;
                let cpu_flags: Flags = Flags::from_bits_truncate(cpu.state.flags);
                let jump_condition_met: bool = match jmp_code {
                    0x0 => true,
                    0x1 => cpu_flags.contains(Flags::C),
                    0x2 => cpu_flags.contains(Flags::Z),
                    0x3 => cpu_flags.contains(Flags::N),
                    0x4 => !(cpu_flags.contains(Flags::N) | cpu_flags.contains(Flags::Z)),
                    0x5 => cpu_flags.contains(Flags::N) | cpu_flags.contains(Flags::Z), 
                    0x6 => !(cpu_flags.contains(Flags::N)),
                    0x7 => !(cpu_flags.contains(Flags::Z)),
                    0x8 => cpu_flags.contains(Flags::O),
                    0x9 => cpu_flags.contains(Flags::P),
                    0xA => !(cpu_flags.contains(Flags::C) | cpu_flags.contains(Flags::Z)),
                    0xB => !(cpu_flags.contains(Flags::C)), 
                    0xC => !(cpu_flags.contains(Flags::C) | cpu_flags.contains(Flags::Z)),
                    _ => { println!("jmp jump_code error! invalid instruction."); false },
                };

                if jump_condition_met {
                    cpu.state.pc = enc.imm;
                } else {
                    cpu.state.pc = cpu.state.pc+1;
                }

            },
            repr: |enc| {
                let jmp_code: u8 = ((enc.rs1 << 3) as u8) +  enc.rd as u8;
                let jmp_code_str: &str;

                match jmp_code {
                    0x0 => jmp_code_str = "jmp",
                    0x1 => jmp_code_str = "jca",
                    0x2 => jmp_code_str = "jeq",
                    0x3 => jmp_code_str = "jlt",
                    0x4 => jmp_code_str = "jgt",
                    0x5 => jmp_code_str = "jle",
                    0x6 => jmp_code_str = "jge",
                    0x7 => jmp_code_str = "jne",
                    0x8 => jmp_code_str = "jovf",
                    0x9 => jmp_code_str = "jpar",
                    0xA => jmp_code_str = "jgtu",
                    0xB => jmp_code_str = "jgeu",
                    0xC => jmp_code_str = "jleu",
                    _ => jmp_code_str = "jump_code error",
                }
                format!("{} {}",jmp_code_str, enc.imm) },
        });
        m.insert(Opcode::CAI as u8, Operation {
            execute: |enc, cpu| {
                let _out: u32 = cpu.state.reg[enc.rs1 as usize] as u32 & enc.imm as u32;
                cpu.state.flags = gen_flag(true, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out)
            },
            repr: |enc| format!("cai r{}, r{}", enc.rs1, enc.imm),
        });
        m.insert(Opcode::SAR as u8, Operation {
            execute: |enc, cpu| {
                let _out:u32 = match extract(cpu.state.reg[enc.rs1 as usize] as u32, 15, 1) {
                    1 => (cpu.state.reg[enc.rs1 as usize] as u32 >> cpu.state.reg[enc.rs2 as usize] as u32) | (0b1111111111111111 <<  (16  - cpu.state.reg[enc.rs2 as usize])),
                    _ => cpu.state.reg[enc.rs1 as usize] as u32 >> cpu.state.reg[enc.rs2 as usize] as u32,
                };
                
                cpu.state.reg[enc.rd as usize] = _out as u16;
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, cpu.state.reg[enc.rs2 as usize] as u32, _out);
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("sar r{0}, r{1}, r{2}", enc.rd, enc.rs1, enc.rs2),
         });
         m.insert(Opcode::SAI as u8, Operation {
            execute: |enc, cpu| {
                let _out:u32 = match extract(cpu.state.reg[enc.rs1 as usize] as u32, 15, 1) {
                    1 => (cpu.state.reg[enc.rs1 as usize] as u32 >> enc.imm as u32) | (0b1111111111111111 << (16 - enc.imm)),
                    _ => cpu.state.reg[enc.rs1 as usize] as u32 >> enc.imm as u32,
                };
                //
                cpu.state.reg[enc.rd as usize] = _out as u16;                               
                cpu.state.flags = gen_flag(false, cpu.state.reg[enc.rs1 as usize] as u32, enc.imm as u32, _out);
                cpu.state.pc = cpu.state.pc+1; 
            },
            repr: |enc| format!("sai r{0}, r{1}, {2}", enc.rd, enc.rs1, enc.imm),
         });
        m.insert(Opcode::SEX as u8, Operation {
            execute: |enc, cpu|{
                cpu.state.reg[enc.rd as usize] = match extract(cpu.state.reg[enc.rs1 as usize] as u32, 7, 1) {
                    0 => cpu.state.reg[enc.rs1 as usize],
                    _ => cpu.state.reg[enc.rs1 as usize] | 0b1111111100000000,
                    //there is no edge cases. only one other outcome could be "1" but rust does not compile othetr way
                };
                cpu.state.pc = cpu.state.pc + 1; 
            },
            repr: |enc| format!("sex r{}", enc.rs1)
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

fn gen_flag(is_subtract: bool, var_1: u32, var_2: u32, var_out: u32) -> u16 {
    let mut temp_flag = Flags::empty();
    if extract(var_out, 15, 1) == 1 { temp_flag |= Flags::N; }
    
    if extract(var_out, 16, 1) == 1 { temp_flag |= Flags::C; }

    if (extract(var_1, 15, 1) ^ extract(var_2, 15, 1) ^ (is_subtract as u32)) & (extract(var_1, 15, 1) ^ extract(var_out, 15, 1)) == 1 { temp_flag |= Flags::O; }
    

    if var_out as u16 == 0 { temp_flag |= Flags::Z; }

    if var_out.count_ones() % 2 == 0 { temp_flag |= Flags::P; }

    temp_flag.bits()


}


    /* wewy cvool code :pleeding_face:
    var_out ^= var_out >> 8;
    var_out ^= var_out >> 4;
    var_out ^= var_out >> 2;
    var_out ^= var_out >> 1;
    if extract(var_out, 0, 1) == 0 { temp_flag |= Flags::P; }
    */
