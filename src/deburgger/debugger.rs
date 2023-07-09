use console::Term;
use console::style;
use std::collections::HashSet;

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
    watch_list_pc: HashSet<u16>,    //program counter breakpoints   //HashSet boss
}

impl Debugger<'_> {
    pub fn new(cpu: &mut CPU) -> Debugger {
        Debugger { cpu: cpu, state: StatesD::Frozen, watch_list_pc: HashSet::new(), }
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
        //print!("some msg: ");
        let gówno = term.read_char();
        match gówno {
            Ok(ok) => return ok,
            Err(error) => {println!{"error occured: {:?}", error} return 'b'; }
        }
    }

    fn catch_string(&self) -> String { // *niggy wiggy, i should probably add option for int extraction to watch for invalid input but i am too lazy for it rn. i count on that that users does not have severe skill issue.
        //print!("input: ");
        let term: Term = Term::stdout();
        
        
        let input_string = term.read_line();
        match input_string {
            Ok(ok) => return ok,
            Err(error) => {println!{"error occured: {:?}", error} return String::from("0"); }
        }
    }

    fn check_watch(&mut self) -> bool {
        if self.watch_list_pc.contains(&self.cpu.state.pc) { //piotr wawrzyniak
            self.change_state_to(StatesD::Frozen);
        }

        //other watches.

        return false
    }

    fn modify_menu(&mut self) {
        //watchlist print here
        if !self.watch_list_pc.is_empty() {
            println!("{0: <10} |--{1:}--|", "", style("BrkpntPC").yellow() );    //change this to console crate or this other one
            
            //i know how does it look. but trust me, this is the most sane way to do this.
            let mut temp_hashset: HashSet<u16> = HashSet::with_capacity(self.watch_list_pc.capacity());
            let temp_set = self.watch_list_pc.drain();
            for breakpoint_pc in temp_set {
                temp_hashset.insert(breakpoint_pc);
                println!("{: <10} | {: <10} |", "", style(breakpoint_pc).on_magenta(), ); //drain sucks.
            }
            self.watch_list_pc = temp_hashset;

            println!("{0: <10} |____________|", "");
            //println!("{0: <10} |--placehold-|", "");  //future mem breakpoints here
            //println!("{0: <10} |____________|", "");
            println!("");
        }

        println!("p - add Pc breakpoint, m - add Memory breakpoint, d - Delete breakpoint, c - Clear watchlist, b - Back ");
        //let mut x: u16;
        match self.catch_char() {
            'p' => { self.watch_list_pc.insert(self.catch_string().parse::<u16>().unwrap()); println!("") },
            'd' => { println!("p - pc breakpoint, m - mem breakpoint"); match self.catch_char() {
                    'p' => { self.watch_list_pc.remove(&self.catch_string().parse::<u16>().unwrap()); println!(""); }, //niggy wiggy*
                    'm' => todo!(),
                     _ => println!("invalid type"),
                }; println!("") },
            'm' => todo!(),
            //'d' => { self.watch_list_pc.remove(&self.catch_string().parse::<u16>().unwrap()); println!("") },   //sussy wussy. but it works. it's wierd that HashSets need & in wierd places but that's the price to pay for cool opts i guess. i am not going to tryhard and read whole docs on that.
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
        */ //mom plz help me i am scared
    }

    fn choose_state(&mut self) {                
        match self.catch_char() {
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