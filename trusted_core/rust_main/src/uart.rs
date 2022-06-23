// UART driver

use core::fmt::Write;
use core::fmt::Error;

pub struct Uart {
    base_address: usize,
}

impl Write for Uart {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Uart {
            base_address,
        }
    }

    pub fn init(&mut self) {
        let ptr: *mut u8 = self.base_address as *mut u8;
        unsafe {
            // 1. set the word length at line control register (LCR),
            // which is base_addr + 3, write 0b11 to LCR's
            // bit 0 and 1.
            ptr.add(3).write_volatile(0b11);

            // 2. enable the FIFO at FIFO control register (FCR),
            // which is base address + 2, write 0b1 to LCR's bit 0.
            ptr.add(2).write_volatile(0b1);

            // 3. set band rate
            let divisor: u16 = 592;
            let divisor_least: u8 = (divisor & 0xFF).try_into().unwrap();
            let divisor_most: u8 = (divisor >> 8).try_into().unwrap();
            let lcr: u8 = ptr.add(3).read_volatile();
            ptr.add(3).write_unaligned(lcr | 1 << 7);
            ptr.add(0).write_volatile(divisor_least);
            ptr.add(1).write_volatile(divisor_most);
            ptr.add(3).write_volatile(lcr);
        }
    }

    pub fn put(&mut self, c: u8) {
        let ptr: *mut u8 = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    pub fn get(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            }
            else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }

    }
}
