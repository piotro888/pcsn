pub trait Device {
    fn read(&mut self, address: u32, sel: u8) -> u16;
    fn write(&mut self, address: u32, sel: u8, data: u16);
}

pub struct DeviceEntry {
    pub device: Box<dyn Device>,
    pub begin_addr: u32,
    pub end_addr: u32
}

pub struct Bus {
    devices: Vec<DeviceEntry>
}

impl Bus {
    pub fn add_device(&mut self, dev_ent: DeviceEntry) {
         self.devices.push(dev_ent);
    }

    fn find_device(&mut self, addr: u32) -> Option<&mut dyn Device> {
        for dev in &mut self.devices {
            if addr >= dev.begin_addr && addr <= dev.end_addr {
                return Some(dev.device.as_mut())
            }
        }
        None
    }

    pub fn new() -> Bus {
        Bus { devices: vec![] }
    }
}

impl Device for Bus {
    fn read(&mut self, address: u32, sel: u8) -> u16 {
        let dev = self.find_device(address).unwrap(); // TODO: Support bus err respose in some
                                                      // cases and panic in others
        dev.read(address, sel)
    }
    
    fn write(&mut self, address: u32, sel: u8, data: u16) {
        let dev = self.find_device(address).unwrap();
        
        dev.write(address, sel, data) 
    }
}
