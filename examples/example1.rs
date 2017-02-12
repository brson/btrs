extern crate btrs;

use btrs::errors::*;
use btrs::units::*;
use btrs::wal::*;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    let ps = PageSize::new(512);
    let mut wal = Wal::new("./testdb.db", ps)?;
    {
        let mut wal = wal.begin_read()?;
        let mut wal = wal.begin_write()?;
        let mut page = Page::new(ps);
        page.buf_mut()[0] = 1;
        wal.write_page(0, page)?;
        wal.commit()?;
    }
    {
        let mut wal = wal.begin_read()?;
        let page = wal.read_page(0)?.unwrap();
        assert!(page.buf()[0] == 1);
    }

    Ok(())
}

