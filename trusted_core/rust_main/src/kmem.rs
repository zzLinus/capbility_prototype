use crate::mutex::Mutex;

pub static KMEM: Mutex<Kmem> = Mutex::new(Kmem::new());

pub static PATTERN: [u8; 8] = [
    0b1000_0000,
    0b0100_0000,
    0b0010_0000,
    0b0001_0000,
    0b0000_1000,
    0b0000_0100,
    0b0000_0010,
    0b0000_0001,
];

pub const PAGE_NUM: usize = 25600;
pub const PAGE_ADDR: usize = 0x85000000;
pub const PAGE_SIZE: usize = 0x1000;

#[allow(dead_code)]
pub struct Kmem {
    pub base_addr: usize,
    bitmap: [u8; PAGE_NUM / 8],
    front: usize,
    num_before_front: usize,
    total: usize,
}

#[allow(dead_code)]
impl Kmem {
    pub const fn new() -> Kmem {
        Self {
            base_addr: PAGE_ADDR,
            bitmap: [0b1111_1111; PAGE_NUM / 8],
            front: 0,
            num_before_front: 0,
            total: PAGE_NUM,
        }
    }

    fn num2coordinate(&mut self, num: usize) -> (usize, usize) {
        (num / 8, num % 8)
    }

    fn find_begin(&mut self, size: usize) -> Option<usize> {
        let mut sum = 0;
        let mut begin = self.front;
        let (mut row, mut col) = self.num2coordinate(self.front);
        while begin <= PAGE_NUM - size {
            if self.bitmap[row] & PATTERN[col] > 0 {
                sum += 1;
                if sum == size {
                    return Some(begin);
                }
            } else {
                begin = begin + sum + 1;
                sum = 0;
            }
            col += 1;
            if col > 7 {
                col = 0;
                row += 1;
            }
        }
        None
    }

    fn set_bits(&mut self, begin: usize, size: usize, flag: bool) {
        let (mut row, mut col) = self.num2coordinate(begin);
        if flag {
            for _i in 0..size {
                self.bitmap[row] = self.bitmap[row] | PATTERN[col];
                col += 1;
                if col > 7 {
                    col = 0;
                    row += 1;
                }
            }
        } else {
            for _i in 0..size {
                self.bitmap[row] = self.bitmap[row] ^ PATTERN[col];
                col += 1;
                if col > 7 {
                    col = 0;
                    row += 1;
                }
            }
        }
    }

    pub fn palloc(&mut self, size: usize) -> Option<usize> {
        let begin = self.find_begin(size);
        match begin {
            Some(x) => {
                self.set_bits(x, size, false);
                self.total = self.total - size;
                return Some(self.base_addr + x * PAGE_SIZE);
            }
            None => return None,
        }
    }

    pub fn pfree(&mut self, addr: usize, size: usize) {
        let begin = (addr - self.base_addr) / PAGE_SIZE;
        self.set_bits(begin, size, true);
        self.total = self.total + size;
    }

    pub fn format_print(&self) {
        for i in 0..PAGE_NUM / 8 {
            println!("{:#010b}", self.bitmap[i]);
        }
    }

    pub fn get_bitmap(&self, row: usize) -> u8 {
        return self.bitmap[row];
    }
}

#[cfg(kernel_test)]
use crate::test_framework;

#[cfg(kernel_test)]
use crate::mutex::MutexGuard;

#[cfg(kernel_test)]
use rand::RngCore;

#[cfg(kernel_test)]
pub fn test_kmem() -> test_framework::TestResult {
    let mut result = test_framework::TestResult {
        passed: 0,
        failed: 0,
    };
    let mut kmem = KMEM.lock();
    println!("test kmem:");
    if test_palloc_pfree_sequence(&mut kmem) {
        result.passed += 1;
        println!("passed!");
    } else {
        result.failed += 1;
        println!("failed!");
    }
    if test_palloc_pfree_random(&mut kmem) {
        result.passed += 1;
        println!("passed!");
    } else {
        result.failed += 1;
        println!("failed!");
    }

    result
}

#[cfg(kernel_test)]
//Check whether the status of the size page starting from begin is flag
pub fn check_bitmap(kmem: &mut MutexGuard<Kmem>, begin: usize, size: usize, flag: bool) -> bool {
    let (mut row, mut col) = kmem.num2coordinate(begin);
    for _i in 0..size {
        if (kmem.get_bitmap(row) & PATTERN[col] == PATTERN[col]) == flag {
            col += 1;
            if col > 7 {
                col = 0;
                row += 1;
            }
            continue;
        } else {
            return false;
        }
    }
    true
}

#[cfg(kernel_test)]
pub fn test_palloc_pfree_sequence(kmem: &mut MutexGuard<Kmem>) -> bool {
    println!("test_palloc_free_sequence");
    for i in 1..100 {
        let size = i;
        let begin = kmem.find_begin(size);
        match begin {
            Some(x) => match kmem.palloc(size) {
                Some(y) => {
                    if !check_bitmap(kmem, x, size, false) {
                        return false;
                    }
                    kmem.pfree(y, size);
                    if !check_bitmap(kmem, x, size, true) {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            },
            None => {
                return false;
            }
        }
    }
    true
}

#[cfg(kernel_test)]
pub fn test_palloc_pfree_random(kmem: &mut MutexGuard<Kmem>) -> bool {
    let mut rng = rand_pcg::Pcg32::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7);
    println!("test_palloc_pfree_random");
    let front = kmem.find_begin(1).unwrap();
    for _i in 1..100 {
        let random_size = rng.next_u32() as usize;
        let alloc_size: usize = random_size % ((PAGE_NUM - front) / 3);
        let begin = kmem.find_begin(alloc_size).unwrap();
        match kmem.palloc(alloc_size) {
            Some(x) => match kmem.palloc(alloc_size) {
                Some(y) => {
                    kmem.pfree(x, alloc_size);
                    if kmem.find_begin(alloc_size / 2).unwrap() != begin {
                        return false;
                    }
                    if kmem.find_begin(alloc_size).unwrap() != begin {
                        return false;
                    }
                    match kmem.find_begin(alloc_size + 1) {
                        Some(z) => {
                            if z != begin + 2 * alloc_size {
                                return false;
                            }
                        }
                        None => {
                            if (alloc_size + 1) * 3 <= PAGE_NUM - front {
                                return false;
                            }
                        }
                    }
                    kmem.pfree(y, alloc_size);
                }
                None => {
                    return false;
                }
            },
            None => {
                return false;
            }
        }
    }
    true
}
