use crate::devices::bus::Device;

pub struct ROM<'a> {
    mem: &'a[u16] 
}

impl Device for ROM<'_> {
    fn read(&mut self, addr: u32, _sel: u8) -> u16 {
        self.mem[addr as usize]
    }

    fn write(&mut self, _addr: u32, _sel: u8, _data: u16) {
    }
}

impl ROM<'_> {
    pub fn new(content: &[u16])  -> ROM {
        ROM { mem: content }
    }
}

