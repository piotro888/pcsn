mod cpu;
mod devices;
mod support;

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io::Read;

use clap::Parser;

use crate::devices::bus::{Bus, DeviceEntry, Device};
use crate::devices::ram::RAM;
use crate::devices::uart::UART;

fn build_system(prog_init: &[u8], data_init: &[u8]) {
    let mut bus = Bus::new();

    const RAM_START: u32 = 0x10_00000;
    const RAM_END: u32   = 0xff_dffff;

    let mut ram = RAM::with_size((RAM_END-RAM_START) as usize);
    ram.load_at(0x10_00000-RAM_START, prog_init);
    ram.load_at(0x80_00000-RAM_START, data_init);
    bus.add_device(DeviceEntry {begin_addr: RAM_START, end_addr: 0xffdfff, device: Box::new(ram)});

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

#[derive(Parser)]
struct CliArgs {
    /// path of binary file with instructions (loaded to 0x800000)
    prog_bin_path: std::path::PathBuf,
    /// path of binary file with data (loaded to 0x100000)
    data_bin_path: std::path::PathBuf,
}

fn read_file(path: &std::path::PathBuf) -> Vec<u8> {
    let mut buff = Vec::new();
    File::open(path).expect(&format!("Failed to open file {}", path.to_str().unwrap()))
        .read_to_end(&mut buff).expect(&format!("Failed to read file {}", path.to_str().unwrap()));
    buff
}

fn main() {
    let args = CliArgs::parse();

    let prog_buff = read_file(&args.prog_bin_path);
    let data_buff = read_file(&args.data_bin_path);

    build_system(&prog_buff, &data_buff);
}
