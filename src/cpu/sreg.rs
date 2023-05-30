use crate::cpu::cpu::State;

const MMU_SIZE: usize = 16;

pub struct SregCoreState {
    sr1_priv: u16,
    
    sr2_jtr: u16,
    sr2_jtr_buff: u16,
    
    sr3_irq_pc: u16,

    sr5_irq_flags: u16,
    
    sr6_scratch: u16,

    dmmu: [u16; MMU_SIZE],
    immu: [u16; MMU_SIZE],

    coreid: u16,
}

#[allow(non_camel_case_types)]
#[derive(enumn::N)]
#[repr(u16)]
enum SREG {
    PC = 0,
    PRIV,
    JTR,
    IRQ_PC,
    ALU_FL,
    IRQ_FL,
    SCRATCH,
    CPUID,
    COREID,
    IC_INT_SET,
    IC_INT_RESET,
    CORE_DISABLE,
    IMMU = 0x100,
    DMMU = 0x200,
}


const PRIV_PRIV: u16  = 0b0001;
const PRIV_DATPG: u16 = 0b0010;
const PRIV_IRQ: u16   = 0b0100;

const JTR_INSTPG: u16 = 0b001; 

impl SregCoreState {
    pub fn new(coreid: u16) -> SregCoreState {
        SregCoreState {
            sr1_priv: PRIV_PRIV, 
            sr2_jtr: JTR_INSTPG, sr2_jtr_buff: JTR_INSTPG,
            sr3_irq_pc: 0, sr5_irq_flags: 0, sr6_scratch: 0,
            immu: [0x7fe, 0x7ff, 0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            dmmu: [0; MMU_SIZE],
            coreid
        }
    }

    pub fn write(&mut self, addr: u16, data: u16, cpu_state: &mut State) {
        match SREG::n(addr) {
            Some(SREG::PC)  => {
                cpu_state.pc = data; 
            }
            Some(SREG::PRIV) => {
                if (self.sr1_priv & PRIV_PRIV) != 0 {
                    self.sr1_priv = data;
                }
            }
            Some(SREG::JTR) => {
                if (self.sr1_priv & PRIV_PRIV) != 0 {
                    self.sr2_jtr_buff = data & ((1<<3)-1);
                }
            }
            Some(SREG::IRQ_PC) => {
                self.sr3_irq_pc = data;
            }
            Some(SREG::ALU_FL) => {
                cpu_state.flags = data & ((1<<5)-1);
            }
            Some(SREG::SCRATCH) => {
                self.sr6_scratch = data;
            }
            _ => {}  
        }

        if addr >= SREG::IMMU as u16 && addr < SREG::IMMU as u16 + MMU_SIZE as u16 {
            self.immu[addr as usize - SREG::IMMU as usize] = data & ((1<<12)-1);
        }
        if addr >= SREG::DMMU as u16 && addr < SREG::DMMU as u16 + MMU_SIZE as u16 {
            self.immu[addr as usize - SREG::DMMU as usize] = data & ((1<<13)-1);
        }
    }

    pub fn read(&mut self, addr: u16, cpu_state: &State) -> u16 {
        match SREG::n(addr) {
            Some(SREG::PC)  => cpu_state.pc, 
            Some(SREG::PRIV) => self.sr1_priv,
            Some(SREG::JTR) => self.sr2_jtr,
            Some(SREG::IRQ_PC) => self.sr3_irq_pc,
            Some(SREG::ALU_FL) => cpu_state.flags,
            Some(SREG::IRQ_FL) => self.sr5_irq_flags,
            Some(SREG::SCRATCH) => self.sr6_scratch,
            Some(SREG::CPUID) =>  0b1011_0000_0011_0011,
            Some(SREG::COREID) => self.coreid,
            _ => 0 
        }
    }
}

const IMMU_DISABLED_MASK: u32 = 0x80_0000;
const DMMU_DISABLED_MASK: u32 = 0x10_0000;

impl SregCoreState {
    pub fn jtr_trig(&mut self) {
        self.sr2_jtr = self.sr2_jtr_buff;
    }

    pub fn immu_translate(&self, addr: u16) -> u32 {
        if (self.sr2_jtr & JTR_INSTPG) == 0 {
            return IMMU_DISABLED_MASK | addr as u32;  
        }
        let addr_low: u32 = (addr & ((1<<12)-1)) as u32;
        let page: u32 = self.immu[(addr>>24) as usize] as u32;
        (page<<12) | addr_low
    }

    pub fn dmmu_translate(&self, addr: u16) -> u32 {
        // pre-wb address, but 16 bit addressed -> MSB is clear
        if (self.sr1_priv & PRIV_DATPG) == 0 {
            return DMMU_DISABLED_MASK | addr as u32;  
        }
        let addr_low: u32 = (addr & ((1<<11)-1)) as u32;
        let page: u32 = self.immu[(addr>>23) as usize] as u32;
        (page<<11) | addr_low
    }
}

