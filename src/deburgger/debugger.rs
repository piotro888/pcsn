use console::Term;
use console::style;

use crate::cpu::cpu::CPU;

#[derive(PartialEq, Eq)]
#[derive(Debug, Copy, Clone)]
pub enum StatesD
{                
    Frozen,     //  cpu is halted. debuger is waiting for further commands.
    Continuous, //C cpu ticks without limits. (in future there will be a way to enter "Frozen" mode from "Continuous" mode)
    Step,       //S cpu does 1 tick and enters "Frozen" mode
    OnWatch,    //W deburger will ask for a watch value, and then cpu ticks until pc == self.watch. then cpu enters "Frozen" mode.
}

pub struct Debugger<'a>
{
    cpu: &'a mut CPU,
    pub state: StatesD,
    watch: u16,
}

impl Debugger<'_>
{
    pub fn new(cpu: &mut CPU) -> Debugger
    {
        Debugger { cpu: cpu, state: StatesD::Frozen, watch: 0, }
    }

    pub fn debuger_loop(&mut self)
    {
        loop {        
                match self.state {
                StatesD::Continuous => self.do_a_cpu_tick(),
                StatesD::Frozen => self.choose_state(),
                StatesD::OnWatch => { self.do_a_cpu_tick(); if self.cpu.state.pc == self.watch { self.change_state_to(StatesD::Frozen); }; },
                StatesD::Step => { self.do_a_cpu_tick(); self.state = StatesD::Frozen},
            }
        }
    }

    fn do_a_cpu_tick(&mut self)
    {
        self.cpu.tick();
        /*      TODO: do something with this scary looking thing below
            if irqc.borrow().active() {
                self.cpu.sregs.add_interrupt(cpu::sreg::IRQF_EXT);
            }
        */
    }

    fn choose_state(&mut self)
    {                
        //user input handler here:
        let term = Term::stdout();
        let state_char: char = term.read_char().unwrap();
                                //                    ^^^ koniec kariery

        match state_char {
            's' => self.change_state_to(StatesD::Step), 
            'w' => self.change_state_to(StatesD::OnWatch),
            'c' => self.change_state_to(StatesD::Continuous),
            _ => println!("unknown / no command"),
        }

    }
    
    fn change_state_to(&mut self, state_i: StatesD )
    {
        self.state = state_i;
        
        if self.state == StatesD::OnWatch
        {
            println!("enter the address of instruction to watch: ");
            let term = Term::stdout();
            self.watch = (term.read_line().unwrap()).parse::<u16>().expect("piotr wawrzyniak a nie u16"); //reading line and converting it to u16. kind of unsafe. for now...
        }
        println!("deburger mode: {:?}", style(self.state).cyan());
    }
    
}