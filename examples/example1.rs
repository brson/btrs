#![allow(unused)]

extern crate btrs;

use std::fs;
use btrs::errors::*;
use btrs::units::*;
use btrs::wal::*;
use btrs::page::*;

fn main() {
    run().unwrap();
}

fn run() -> Result<()> {
    fs::remove_file("./testdb.wal")?;
    fs::remove_file("./testdb.shm")?;
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
    wal.dump();
    {
        let mut wal = wal.begin_read()?;
        let page = wal.read_page(0)?.unwrap();
        assert!(page.buf()[0] == 1);
    }
    wal.dump();
    {
        let wal = wal.begin_checkpoint()?;
        wal.next_epoch()?;
    }
    println!("checkpoint");
    wal.dump();
    {
        let mut wal = wal.begin_read()?;
        let mut wal = wal.begin_write()?;
        let mut page = Page::new(ps);
        page.buf_mut()[0] = 1;
        wal.write_page(1, page)?;
        let mut page = Page::new(ps);
        page.buf_mut()[0] = 1;
        wal.write_page(2, page)?;
        wal.commit()?;
    }
    {
        let mut wal = wal.begin_read()?;
        let mut wal = wal.begin_write()?;
        let mut page = Page::new(ps);
        page.buf_mut()[0] = 2;
        wal.write_page(2, page)?;
        wal.commit()?;
    }
    wal.dump();
    {
        let mut wal = wal.begin_read()?;
        let page = wal.read_page(2)?.unwrap();
        assert!(page.buf()[0] == 2);
    }
    wal.dump();
    {
        let wal = wal.begin_checkpoint()?;
        for page in wal.pages() {
            println!("replaying page {}", page);
            let _page = wal.read_page(*page)?;
        }
        wal.next_epoch()?;
    }
    wal.dump();
    {
        wal.begin_read()?;
    }
    wal.dump();

    Ok(())
}

