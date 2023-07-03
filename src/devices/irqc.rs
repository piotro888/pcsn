use crate::devices::bus::Device;

pub struct Irqc {
    irq_mask: u16,
    irq_active: u16
}

impl Device for Irqc {
    fn read(&mut self, addr: u32, _sel: u8) -> u16 {
        match addr {
            0b10 => self.irq_mask,
            _ => self.irq_mask & self.irq_active,
        }
    }

    fn write(&mut self, addr: u32, _sel: u8, data: u16) {
        match addr {
            0b1 => { self.irq_active = self.irq_active & !data; },
            0b10 => { self.irq_mask = data },
            _ => {},
        };
    }
}

impl Irqc {
    pub fn trigger(&mut self, code: u16) {
        self.irq_active |= code;
    }

    pub fn active(&self) -> bool {
        (self.irq_active & self.irq_mask) != 0
    }

    pub fn new() -> Irqc {
        Irqc { irq_mask: 0, irq_active: 0 }
    }
}
