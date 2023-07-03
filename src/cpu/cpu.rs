use crate::cpu::instr::Encoding;
use crate::cpu::sreg::SregCoreState;
use crate::devices::bus::{Bus, Device};
use super::instr::execute;

pub struct State {
    pub reg: [u16; 8],
    pub pc: u16,
    pub flags: u16,
}

pub struct CPU {
    pub state: State,
    pub sregs: SregCoreState,

    bus: Bus // TODO: remove mut and make bus a pointer??
}

impl CPU {
    fn data_wb_addr(&self, cpu_addr: u16, word: bool) -> (u32, u8) {
        let wb_adr = self.sregs.dmmu_translate(cpu_addr>>1);
        let sel = if word { 0b11 } else { 0b01 << (cpu_addr&1) };
        return (wb_adr, sel);

    }

    pub fn read(&mut self, cpu_addr: u16, word: bool) -> u16 {
        let (wb_adr, wb_sel) = self.data_wb_addr(cpu_addr, word);
        let val = self.bus.read(wb_adr, wb_sel);

        if word {
            val
        } else {
            (val>>((cpu_addr&1)*8)) & 0xff
        }
    }

    pub fn write(&mut self, cpu_addr: u16, word: bool, data: u16) {
        let (wb_adr, wb_sel) = self.data_wb_addr(cpu_addr, word);
        self.bus.write(wb_adr, wb_sel, data)
    }

    pub fn fetch(&mut self) -> u32 {
        // in ppcpu, icache requests lines from wb 16 bit addresses, that are translated later
        let base_addr = self.sregs.immu_translate(self.state.pc<<1);
        println!("immu {:#06x} -> {:#08x} (8a{:#08x})", self.state.pc<<1, base_addr, base_addr<<1);
        let low_part = self.bus.read(base_addr, 0b11) as u32;
        let high_part = self.bus.read(base_addr+1, 0b11) as u32;

        (high_part << 16) | low_part
    }

    pub fn execute(&mut self, instr: u32) {
        println!("{:#010x}", instr);
        for i in 0..8 {
            print!("r{}: {:#06x} ", i, self.state.reg[i]);
        }
        println!("");
        let encoding = Encoding::from_raw(instr);
        execute(&encoding, self);
    }

}

impl CPU {
    pub fn tick(&mut self) {
        let insn = self.fetch();
        self.execute(insn);

        self.sregs.interrupt(&mut self.state)
    }

    pub fn new(bus: Bus, coreid: u16) -> CPU {
       CPU {state: State::new(), sregs: SregCoreState::new(coreid), bus: bus} 
    }
}

impl State {
    pub fn new() -> State {
        State {reg: [0;8], pc: 0, flags: 0}
    }
}
