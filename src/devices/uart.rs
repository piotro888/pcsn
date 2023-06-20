use std::io::Write;

use crate::support::tty::Pty;
use crate::devices::bus::Device;

pub struct UART {
    pub pty: Pty,
    
    last_read: u8,
    last_read_pending: bool
}

const STATUS_ADDR: u32 = 0x0;
const RX_ADDR: u32 = 0x1;
const TX_ADDR: u32 = 0x2;

impl Device for UART {
    fn write(&mut self, address: u32, _sel: u8, data: u16) {
        if address == TX_ADDR {
            print!("txx{}", data as u8 as char);
            self.pty.master_write_file.write(&[data as u8]).unwrap();
        }
    }

    fn read(&mut self, address: u32, _sel: u8) -> u16 {
        match address {
            STATUS_ADDR => {
                // check if new value is available and share it to reading
                if !self.last_read_pending { // peeking is not possible between calls, so this workaround :(
                    let read = self.pty.master_reciever.try_recv();
                    if read.is_ok() {
                        self.last_read = read.unwrap();
                        self.last_read_pending = true;
                    }
                }
                
                let tx_ready = 1;

                self.last_read_pending as u16 | (tx_ready<<1)
            },
            RX_ADDR => {
                if !self.last_read_pending {
                    // try reading new value
                    self.last_read = self.pty.master_reciever.try_recv().unwrap_or(self.last_read);
                }
                self.last_read_pending = false;
                self.last_read as u16
            }
            _ => 0
        }
    }
}

impl UART {
    pub fn new() -> UART {
        let pty = Pty::open().expect("Failed to open PTY terminal pair");
        UART { pty, last_read: 0, last_read_pending: false }
    }
}
