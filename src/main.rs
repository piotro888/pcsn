
mod cpu;
mod devices;
mod support;

#[macro_use]
extern crate lazy_static;


use core::time;
use std::thread;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

use clap::Parser;

use crate::devices::bus::{Bus, DeviceEntry, Device};
use crate::devices::irqc::Irqc;
use crate::devices::ram::RAM;
use crate::devices::rom::ROM;
use crate::devices::sd::SD;
use crate::devices::uart::UART;
use crate::devices::timer::Timer;

use crate::cpu::cpu::CPU;

fn build_system(prog_init: &[u8], data_init: &[u8], sd_file: File) {
    let mut bus = Bus::new();

    const RAM_START: u32 = 0x10_0000;
    const RAM_END: u32   = 0xff_dfff;

    let mut ram = RAM::with_size((RAM_END-RAM_START) as usize);
    ram.load_at(0x80_0000-RAM_START, prog_init);
    ram.load_at(0x10_0800-RAM_START, data_init);
    bus.add_device(DeviceEntry {begin_addr: RAM_START, end_addr: RAM_END, device: Rc::new(RefCell::new(ram))});

    let serial = UART::new();
    serial.pty.spawn_term();
    bus.add_device(DeviceEntry {begin_addr: 0x002000, end_addr: 0x002002, device: Rc::new(RefCell::new(serial))});

    let boot_rom = ROM::new(&BOOTJUMP_ROM);
    bus.add_device(DeviceEntry { device: Rc::new(RefCell::new(boot_rom)), begin_addr: 0xff_e000, end_addr: 0xff_e005 });
    thread::sleep(time::Duration::from_millis(100)); // TODO: wait for xterm lanuch to not miss serial out

    let irqc = Rc::new(RefCell::new(Irqc::new()));
    bus.add_device(DeviceEntry { device: Rc::clone(&irqc) as Rc<RefCell<dyn Device>>, begin_addr: 0x00200c, end_addr: 0x00200e });
    
    let timer = Rc::new(RefCell::new(Timer{}));
    bus.add_device(DeviceEntry { device: Rc::clone(&timer) as Rc<RefCell<dyn Device>>, begin_addr: 0x002008, end_addr: 0x00200a });
    
    let spi_sd = SD::new(sd_file);
    bus.add_device(DeviceEntry { device: Rc::new(RefCell::new(spi_sd)) as Rc<RefCell<dyn Device>>, begin_addr: 0x002010, end_addr: 0x002014 });

    let mut cpu = CPU::new(bus, 0);

    println!("init done");
    loop {
        cpu.tick();
        
        if irqc.borrow().active() {
            cpu.sregs.add_interrupt(cpu::sreg::IRQF_EXT);
        }
    }
}

const BOOTJUMP_ROM: [u16; 6] = [
     0x0004, // ldi r0, 0
     0x0000,
     0x0011, // srs r0, 2
     0x0002,
     0x000E, // jmp 0
     0x0000,
];

#[derive(Parser)]
struct CliArgs {
    /// path of binary file with instructions (loaded to 0x800000)
    prog_bin_path: std::path::PathBuf,
    /// path of binary file with data (loaded to 0x100000)
    data_bin_path: std::path::PathBuf,
    /// path of sd card image file
    sd_img_path: std::path::PathBuf,
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
    let sd_img = File::open(args.sd_img_path).expect("Failed to open SD image file");

    build_system(&prog_buff, &data_buff, sd_img);
}
