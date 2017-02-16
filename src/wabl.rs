use page::Page;
use errors::*;
use wal::*;
use std::path::Path;
use std::convert::AsRef;
use page_store::PageStore;

pub struct Wabl {
    ps: PageStore,
    wal: Wal,
}

pub struct ReadWabl<'a> {
    ps: &'a mut PageStore,
    wal: ReadWal<'a>
}

pub struct WriteWabl<'a> {
    ps: &'a mut PageStore,
    wal: WriteWal<'a>,
}

impl Wabl {
    fn new<P: AsRef<Path>>(p: &P) -> Result<Wabl> { panic!() }

    fn begin_read(&mut self) -> Result<ReadWabl> {
        Ok(ReadWabl {
            ps: &mut self.ps,
            wal: self.wal.begin_read()?,
        })
    }

    fn checkpoint(&mut self) -> Result<()> {
        let mut wal = self.wal.begin_checkpoint()?;

        for page_num in wal.pages() {
            let page = wal.read_page(*page_num)?
                           .expect("checkpoint missing page");
            self.ps.write_page(*page_num, page)?;
        }

        self.ps.sync()?;
        wal.next_epoch()
    }
}

impl<'a> ReadWabl<'a> {
    fn read_page(&mut self, i: PageNum) -> Result<Page> {
        if let Some(p) = self.wal.read_page(i)? {
            return Ok(p);
        }

        self.ps.read_page(i)
    }

    fn begin_write(self) -> Result<WriteWabl<'a>> {
        Ok(WriteWabl {
            ps: self.ps,
            wal: self.wal.begin_write()?,
        })
    }
}

impl<'a> WriteWabl<'a> {
    fn read_page(&mut self, i: PageNum) -> Result<Page> {
        if let Some(p) = self.wal.read_page(i)? {
            return Ok(p);
        }

        self.ps.read_page(i)
    }

    fn write_page(&mut self, i: PageNum, b: Page) -> Result<()> {
        self.wal.write_page(i, b)
    }

    fn commit(self) -> Result<()> {
        self.wal.commit()
    }

    fn rollback(self) -> Result<()> {
        self.wal.rollback()
    }
}
