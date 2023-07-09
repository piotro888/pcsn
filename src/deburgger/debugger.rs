use console::Term;
use console::style;

use crate::cpu::cpu::CPU;

#[derive(PartialEq, Eq)]
#[derive(Debug, Copy, Clone)]
enum StatesD {                
    Frozen,     //  cpu is halted. debuger is waiting for further commands.
    Continuous, //C cpu ticks without limits. (in future there will be a way to enter "Frozen" mode from "Continuous" mode)
    Step,       //S cpu does 1 tick and enters "Frozen" mode
    OnWatch,    //W cpu ticks until it finds watched value. then cpu enters "Frozen" mode.
    Modify,     //M add/delete breakpoints to watchlist.
}

pub struct Debugger<'a> {
    cpu: &'a mut CPU,
    state: StatesD,
    watch_list_pc: Vec<u16>,    //program counter breakpoints   //probably will migrate to HashSet soon
}

impl Debugger<'_> {
    pub fn new(cpu: &mut CPU) -> Debugger {
        Debugger { cpu: cpu, state: StatesD::Frozen, watch_list_pc: Vec::new(), }
    }

    pub fn debuger_loop(&mut self) {
        println!("deburger options: w - set watch, s - step mode, m - watchlist menu, c - continuous mode(debugger off, you cannot change it later!)");
        loop {        
                match self.state {
                StatesD::Continuous => self.do_a_cpu_tick(),
                StatesD::Frozen     => self.choose_state(),
                StatesD::OnWatch    => { self.do_a_cpu_tick(); if self.check_watch() { self.change_state_to(StatesD::Frozen) }; },
                StatesD::Step       => { self.do_a_cpu_tick(); self.state = StatesD::Frozen},
                StatesD::Modify     => self.modify_menu(),
            }
        }
    }

    fn catch_char(&self) -> char {
        let term: Term = Term::stdout();
        let gówno = term.read_char();
        match gówno {
            Ok(ok) => return ok,
            Err(error) => {println!{"error occured: {:?}", error} return 'b'; }
        }
    }

    fn catch_string(&self) -> String {
        let term: Term = Term::stdout();
        let gówno = term.read_line();
        print!{"input value: "}
        match gówno {
            Ok(ok) => return ok,
            Err(error) => {println!{"error occured: {:?}", error} return String::from("0"); }
        }
    }

    fn check_watch(&mut self) -> bool {
        for breakpoint_pc in &self.watch_list_pc {
            if self.cpu.state.pc == *breakpoint_pc { return true };
        }

        return false
    }

    fn modify_menu(&mut self) {
        //watchlist print here
        if !self.watch_list_pc.is_empty() {
            println!("{0: <10} |-----id-----|---value---|", "");
            
            let mut counter: u32 = 0;
            for breakpoint_pc in &self.watch_list_pc {
                println!("{0: <10} | {1: <10} | {2: <9} |", "", counter, breakpoint_pc);
                counter = counter + 1;
            }
            
            println!("{0: <10} |------------|-----------|", "");
        }

        println!("a - add breakpoint, d - delete breakpoint, c - clear watchlist, b - back ");
        //let mut x: u16;
        match self.catch_char() {
            'a' => { self.watch_list_pc.push(self.catch_string().parse::<u16>().unwrap()); println!("") },// unsafe!!!!
            'd' => { self.watch_list_pc.swap_remove(self.catch_string().parse::<usize>().unwrap()); println!("") }, //Vec.remove / Vec.swap_remove are wierd.
            'c' => self.watch_list_pc.clear(),
            'b' => self.change_state_to(StatesD::Frozen),
            _ => println!("unknown/no command"),
        }

    }

    fn do_a_cpu_tick(&mut self) {
        self.cpu.tick();

        /*      TODO: do something with this scary looking thing below
            if irqc.borrow().active() {
                self.cpu.sregs.add_interrupt(cpu::sreg::IRQF_EXT);
            }
        */
    }

    fn choose_state(&mut self) {                
        //user input handler here:
        let term = Term::stdout();
        let state_char: char = term.read_char().unwrap();
                                //                    ^^^ koniec kariery

        match state_char {
            's' => self.change_state_to(StatesD::Step), 
            'w' => self.change_state_to(StatesD::OnWatch),
            'c' => self.change_state_to(StatesD::Continuous),
            'm' => self.change_state_to(StatesD::Modify),
            _ => println!("unknown / no command"),
        }

    }
    
    fn change_state_to(&mut self, state_i: StatesD ) {
        self.state = state_i;
        println!("deburger mode: {:?}", style(self.state).cyan());
    }
    
}