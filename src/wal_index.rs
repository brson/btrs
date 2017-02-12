use lock::*;
use std::rc::Rc;
use errors::*;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::convert::AsRef;
use memmap::{Mmap, Protection};
use fs2::FileExt;
use std::cell::RefCell;

const MAGIC: u64 = 0x7dab6ca4b28afdee; 
const INDEX_SIZE: u64 = 2 ^ 15;

pub struct WalIndex {
    file: Rc<RefCell<File>>,
    mmap: Mmap,
}

struct Header {
    magic: u64,
}

impl WalIndex {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<WalIndex> {
        let file = OpenOptions::new()
            .read(true).write(true).create(true)
            .open(p.as_ref().with_extension("shm"))?;
        file.allocate(INDEX_SIZE);

        let mmap = Mmap::open(&file, Protection::ReadWrite)?;

        let mut index = WalIndex {
            file: Rc::new(RefCell::new(file)),
            mmap: mmap
        };

        index.init()?;

        Ok(index)
    }

    fn init(&mut self) -> Result<()> {
        self.with_header_mut(&mut |header| {
            let reinit = header.magic != MAGIC;
            if reinit {
                header.magic = MAGIC;
            }

            Ok(())
        })
    }

    fn with_header_mut<R>(&mut self, f: &mut FnMut(&mut Header) -> Result<R>) -> Result<R> {
        let _lock = self.write_lock()?;
        let header: &mut Header = unsafe { &mut * (self.mmap.mut_ptr() as *mut Header) };
        f(header)
    }

    pub fn write_lock(&self) -> Result<ExLock> {
        ExLock::new(self.file.clone())
    }
}
