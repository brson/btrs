use units::PageSize;
use std::iter;

pub struct Page(Vec<u8>);

impl Page {
    pub fn new(size: PageSize) -> Page {
        let size = size.to_u32() as usize;
        let mut buf = Vec::with_capacity(size);
        buf.extend(iter::repeat(0).take(size));
        Page(buf)
    }

    pub fn buf(&self) -> &[u8] {
        &self.0
    }

    pub fn buf_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

