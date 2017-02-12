const MIN_PAGE_SIZE: u32 = 512;
const MAX_PAGE_SIZE: u32 = 0x100000; // 1 MB

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Debug)]
pub struct PageSize(u32);

impl PageSize {
    pub fn new(page_size: u32) -> PageSize {
        assert!(page_size >= MIN_PAGE_SIZE);
        assert!(page_size <= MAX_PAGE_SIZE);
        assert!(page_size.is_power_of_two());

        PageSize(page_size)
    }

    pub fn to_u32(&self) -> u32 { self.0 }
}

