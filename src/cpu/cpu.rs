pub struct State {
    pub reg: [u16; 8],
    pub pc: u16,
    pub flags: u16,
}

pub struct CPU {
    state: State
}
