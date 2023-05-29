use crate::devices::bus::Device;

pub struct RAM {
    mem: Box<[u16]>
}

impl Device for RAM {
    fn read(&mut self, addr: u32, _sel: u8) -> u16 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u32, sel: u8, data: u16) {
        match sel {
            0b00 => {}
            0b01 => { self.mem[addr as usize] = (self.mem[addr as usize]&0xff00) | data; }
            0b10 => { self.mem[addr as usize] = (self.mem[addr as usize]&0x00ff) | (data<<8); }
            0b11 => { self.mem[addr as usize] = data; }
            _    => panic!("usupported sel bits: {sel}")
        }
    }
}

impl RAM {
    pub fn with_size(size: usize) -> RAM {
        RAM { mem: vec![0; size].into_boxed_slice() } 
    }
}
