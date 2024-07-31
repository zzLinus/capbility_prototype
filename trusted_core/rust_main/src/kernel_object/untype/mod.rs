use crate::println;

#[derive(Copy, Clone)]
pub struct UntypedObj {
    pub region: Region,
    pub used: Region,
}

#[derive(Copy, Clone)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

impl UntypedObj {
    pub fn new(start: usize, end: usize) -> Self {
        UntypedObj {
            region: Region {
                start: start,
                end: end,
            },
            used: Region {
                start: 0x0,
                end: 0x0,
            },
        }
    }

    pub fn get_region(&self) {
        println!("start {} end {}", self.region.start, self.region.end)
    }

    pub fn get_watermark(&self) {
        println!("start {} end {}", self.used.start, self.used.end)
    }
}
