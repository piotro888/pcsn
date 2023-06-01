mod cpu;
mod devices;
mod support;

#[macro_use]
extern crate lazy_static;

use crate::devices::bus::{Bus, DeviceEntry, Device};
use crate::devices::ram::RAM;
use crate::devices::uart::UART;

fn build_system() {    
    let mut bus = Bus::new();

    let ram = RAM::with_size(0xffdfff-0x100000);
    bus.add_device(DeviceEntry {begin_addr: 0x100000, end_addr: 0xffdfff, device: Box::new(ram)});

    let serial = UART::new();
    serial.pty.spawn_term();
    bus.add_device(DeviceEntry {begin_addr: 0x002000, end_addr: 0x002002, device: Box::new(serial)});
    
    loop {
        if bus.read(0x002000, 0b11)&1 != 0 {
            let rd = bus.read(0x002001, 0b11);
            bus.write(0x002002, 0b11, rd);
        }
    }
}

fn main() {
    build_system();
}
