use errors::*;
use wal::*;

use std::path::Path;
use std::convert::AsRef;

pub struct PageStore;

pub struct Wabl {
    bs: PageStore,
    wal: Wal,
}

pub struct ReadWabl<'a> {
    bs: &'a mut PageStore,
    wal: ReadWal<'a>
}

pub struct WriteWabl<'a> {
    bs: &'a mut PageStore,
    wal: WriteWal<'a>,
}

impl Wabl {
    fn new<P: AsRef<Path>>(p: &P) -> Result<Wabl> { panic!() }

    fn begin_read(&mut self) -> Result<ReadWabl> {
        Ok(ReadWabl {
            bs: &mut self.bs,
            wal: self.wal.begin_read()?,
        })
    }

    fn checkpoint(&mut self) -> Result<()> { panic!() }
}

impl<'a> ReadWabl<'a> {
    fn read_page(&mut self, i: PageNum) -> Result<Page> { panic!() }

    fn begin_write(self) -> Result<WriteWabl<'a>> {
        Ok(WriteWabl {
            bs: self.bs,
            wal: self.wal.begin_write()?,
        })
    }
}

impl<'a> WriteWabl<'a> {
    fn read_page(&mut self, i: PageNum) -> Result<Page> { panic!() }

    fn write_page(&mut self, i: PageNum, b: Page) -> Result<()> { panic!() }

    fn push_page(&mut self, b: Page) -> Result<PageNum> { panic!() }

    fn commit(self) -> Result<()> { panic!() }

    fn rollback(self) -> Result<()> { panic!() }
}
