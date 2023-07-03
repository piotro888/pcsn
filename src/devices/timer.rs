use super::bus::Device;

pub struct Timer {

}

impl Device for Timer {
    fn read(&mut self, addr: u32, _sel: u8) -> u16 {
       0 
    }

    fn write(&mut self, addr: u32, sel: u8, data: u16) {
    }
}
