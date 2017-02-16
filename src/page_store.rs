use fs2::FileExt;
use page::Page;
use wal::PageNum;
use std::path::Path;
use lock::ExLock;
use errors::*;
use units::PageSize;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use byteorder::*;

pub struct PageStore {
    file: Rc<RefCell<File>>,
    page_size: PageSize,
}

const MAGIC: u64 = 0xee2e85c62ff153c8;
const HEADER_SIZE: u32 = 100;

struct Header {
    magic: u64,
    page_size: PageSize,
}

impl PageStore {
    fn new<P: AsRef<Path>>(p: &P, page_size: PageSize) -> Result<PageStore> {
        let file = OpenOptions::new()
            .read(true).write(true).create(true)
            .open(p.as_ref().with_extension("db"))?;

        let mut page_store = PageStore {
            file: Rc::new(RefCell::new(file)),
            page_size: page_size,
        };

        page_store.init(page_size)?;

        Ok(page_store)
    }

    fn init(&mut self, page_size: PageSize) -> Result<()> {
        let lock = ExLock::new(self.file.clone())?;
        panic!()
    }

    fn read_header(&mut self, lock: &ExLock) -> Result<Option<Header>> {
        panic!()
    }

    fn seek_to_page(&mut self, n: PageNum) -> Result<()> {
        let offset = self.page_size.to_u32() as u64 * n as u64;
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }

    pub fn read_page(&mut self, n: PageNum) -> Result<Page> {
        self.seek_to_page(n)?;
        let mut page = Page::new(self.page_size);
        let mut file = self.file.borrow_mut();
        file.read_exact(page.buf_mut())?;
        Ok(page)
    }

    pub fn write_page(&mut self, n: PageNum, p: Page) -> Result<()> {
        self.seek_to_page(n)?;
        let mut file = self.file.borrow_mut();
        file.write_all(p.buf())?;
        Ok(())
    }

    pub fn resize_at_least(&mut self, n: PageNum) -> Result<()> {
        let min_len = n as u64 * self.page_size.to_u32() as u64;
        let mut file = self.file.borrow_mut();
        file.allocate(min_len)?;
        Ok(())
    }

    pub fn sync(&mut self) -> Result<()> {
        self.file.borrow_mut().sync_data()?;
        Ok(())
    }
}
