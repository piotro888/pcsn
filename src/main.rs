mod cpu;
mod devices;

#[macro_use]
extern crate lazy_static;

use crate::devices::bus::{Bus, DeviceEntry};
use crate::devices::ram::RAM;

fn build_system() {    
    let mut bus = Bus::new();

    let ram = RAM::with_size(1<<24);
    bus.add_device(DeviceEntry{begin_addr: 0, end_addr: (1<<24)-1, device: Box::new(ram)});
}

fn main() {
    build_system();
}
