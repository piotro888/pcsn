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
    pub fn with_size(length: usize) -> RAM {
        RAM { mem: vec![0; length].into_boxed_slice() }
    }

    pub fn load_at(&mut self, addr: u32, data: &[u8]) {
        let mut le_data = vec![0; data.len()/2];
        for (pos, ent) in le_data.iter_mut().enumerate() {
            *ent = u16::from_le_bytes(data[pos*2..pos*2+1].try_into().unwrap())
        }

        self.mem[addr as usize..addr as usize+le_data.len()].copy_from_slice(&le_data);

        if data.len() % 2 != 0 {
            self.mem[addr as usize + le_data.len()] &= 0xff00;
            self.mem[addr as usize + le_data.len()] |= *data.last().unwrap() as u16;
        }
    }
}
