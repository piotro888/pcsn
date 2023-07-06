use console::*; //TODO: choose what i really need
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
    
    pub fn check_watch(&mut self)
    {
        //if field from cpu = watch, then
        if self.watch == self.cpu.state.pc {
            self.state = StatesD::Frozen;
        }
    }

    pub fn debuger_tick(&mut self)
    {
            match self.state {
            StatesD::Continuous => self.cpu.tick(),
            StatesD::Frozen => self.choose_state(),
            StatesD::OnWatch => { if self.cpu.state.pc == self.watch { self.state = StatesD::Frozen }; self.cpu.tick(); },
            StatesD::Step => { self.cpu.tick(); self.state = StatesD::Frozen},
            _ => panic!(),  //lol imposible to get that.
        }
        
        /*      TODO: do something with this scary looking thing below
        if irqc.borrow().active() {
            self.cpu.sregs.add_interrupt(cpu::sreg::IRQF_EXT);
        }
        */
        
    }

    pub fn choose_state(&mut self)
    {                
        //user input handler here:
        let term = Term::stdout();
        let mut state_char: char = term.read_char().unwrap();
        println!("{}", state_char); //debug line       ^^^ koniec kariery

        match state_char {//for some reason deburger only works on mode that is on the line below. to test other modes you have to change StatesD::___ to desired mode.
            s => self.change_state_to(StatesD::OnWatch), //change later to "Step"!!!
            w => self.change_state_to(StatesD::OnWatch),
            c => self.change_state_to(StatesD::Continuous),
            _ => panic!() //println!("unknown / no command, C mode activated."),
        }//        ^ only for the bug seeking :)

    }
    
}