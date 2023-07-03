use std::rc::Rc;
use std::cell::RefCell;

pub trait Device {
    fn read(&mut self, address: u32, sel: u8) -> u16;
    fn write(&mut self, address: u32, sel: u8, data: u16);
}

pub struct DeviceEntry {
    pub device: Rc<RefCell<dyn Device>>,
    pub begin_addr: u32,
    pub end_addr: u32,
}

pub struct Bus {
    devices: Vec<DeviceEntry>
}

impl Bus {
    pub fn add_device(&mut self, dev_ent: DeviceEntry) {
         self.devices.push(dev_ent);
    }

    fn find_device(&mut self, addr: u32) -> Option<&mut DeviceEntry> {
        for dev in &mut self.devices {
            if addr >= dev.begin_addr && addr <= dev.end_addr {
                return Some(dev)
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
        print!("Bus read addr={:#08x}, sel={}", address, sel);
        let dev = self.find_device(address).unwrap(); // TODO: Support bus err respose in some
                                                      // cases and panic in others
        let r = dev.device.borrow_mut().read(address-dev.begin_addr, sel);
        println!("  resp={:#05x}", r);
        r
    }
     
    fn write(&mut self, address: u32, sel: u8, data: u16) {
        println!("Bus write addr={:#08x}, sel={}, data={}", address, sel, data);
        let dev = self.find_device(address).unwrap();
        dev.device.borrow_mut().write(address-dev.begin_addr, sel, data) 
    }
}
