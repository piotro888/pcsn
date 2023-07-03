use std::{fs::File, collections::VecDeque, io::{Seek, Read}};

use super::bus::Device;

pub struct SD {
    file: File,

    command_buf: [u8; 6],
    response: VecDeque<u8>,
    curr_resp: u8
}

#[allow(non_camel_case_types)]
#[derive(enumn::N)]
#[repr(u8)]
enum Commands {
    ACMD_PRE_CMD55 = 0x40 | 55,
    ACMD41 = 0x40 | 41,
    CMD58 = 0x40 | 58,
    CMD16 = 0x40 | 16,
    CMD0 = 0x40,
    CMD8 = 0x40 | 8,
    CMD17 = 0x40 | 17,
}


// Simple SD card SPI mode mock. It is not a full implementation, just to satisfy piOS interface.
// TODO: Make SPI device emulator that only calls the device. For now emulate SPI dev too

impl Device for SD {
    fn read(&mut self, addr: u32, _sel: u8) -> u16 {
        if addr != 1 {
            return 0;
        }
        dbg!(self.curr_resp);
        self.curr_resp as u16
    }

    fn write(&mut self, addr: u32, _sel: u8, data: u16) {
        if addr != 0 {
            return;
        }

        self.command_buf.rotate_left(1);
        self.command_buf[self.command_buf.len()-1] = data as u8;
        
        dbg!(data);
        self.curr_resp = self.response.pop_front().unwrap_or(0xff);

        if self.command_buf[0] != 0xff {
            self.process_cmd();
            
            // invalidate completed command
            self.command_buf = [0xff;6];
        }
    }
}

impl SD {
    pub fn new(f: File) -> SD {
        SD {file: f, command_buf: [0xff;6], response: VecDeque::new(), curr_resp: 0xff }
    }

    fn process_cmd(&mut self) {
        match Commands::n(self.command_buf[0]) {
            Some(Commands::ACMD_PRE_CMD55) => {
                self.response.push_back(0x1); // STATUS_IDLE
            },
            Some(Commands::ACMD41) => {
                dbg!(self.command_buf);
                if self.command_buf[1] == 0x40 { // ARG_HC
                    self.response.push_back(0x0); // STATUS_NULL - initialized
                }
            },
            Some(Commands::CMD58) => {
                self.response.push_back(0x0); // STATUS_NULL
                self.response.push_back(0x80); // OCR_0_HC
                self.response.push_back(0x10); // OCR_1_3V3
                self.response.push_back(0x0);
                self.response.push_back(0x0);
            },
            Some(Commands::CMD16) => {
                if self.command_buf[3] != 0x2 || self.command_buf[4] != 0x0 {
                    panic!("Unsupported block size");
                }
                self.response.push_back(0x0); // STATUS_NULL
            },
            Some(Commands::CMD0) => {
                self.response.push_back(0x1); // STATUS_IDLE
                dbg!(self.command_buf);
            },
            Some(Commands::CMD8) => {
                self.response.push_back(0x1); // STATUS_NULL
                self.response.push_back(0x0);
                self.response.push_back(0x0);
                self.response.push_back(0x1); // 3_VOLTAGE
                self.response.push_back(self.command_buf[4]); // check pattern
            },
            Some(Commands::CMD17) => { // The actual READ BLOCK command
                self.response.push_back(0x0); // status
                self.response.push_back(0xff); // data wait
                self.response.push_back(0xfe); // data start token
                
                let page = u32::from_be_bytes(self.command_buf[1..5].try_into().unwrap());
                let seek_res = self.file.seek(std::io::SeekFrom::Start(page as u64 *512));
                dbg!(&seek_res);
                dbg!(page);
                let mut buff = [0 as u8; 512];
                if seek_res.is_ok() {
                    self.file.read(buff.as_mut_slice()).unwrap();
                }
                
                self.response.extend(buff.as_slice());


                self.response.push_back(0x0); // crc not implemented
                self.response.push_back(0x0);
            }
            _ => panic!("Unsupported command")
        }
    }
}
