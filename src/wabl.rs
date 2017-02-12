use errors::*;
use wal::*;

use std::path::Path;
use std::convert::AsRef;

pub struct BlockStore;

pub struct Wabl {
    bs: BlockStore,
    wal: Wal,
}

pub struct ReadWabl<'a> {
    bs: &'a mut BlockStore,
    wal: ReadWal<'a>
}

pub struct WriteWabl<'a> {
    bs: &'a mut BlockStore,
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
    fn read_block(&mut self, i: BlockNum) -> Result<Block> { panic!() }

    fn begin_write(&mut self) -> Result<WriteWabl> {
        Ok(WriteWabl {
            bs: self.bs,
            wal: self.wal.begin_write()?,
        })
    }
}

impl<'a> WriteWabl<'a> {
    fn read_block(&mut self, i: BlockNum) -> Result<Block> { panic!() }

    fn write_block(&mut self, i: BlockNum, b: Block) -> Result<()> { panic!() }

    fn push_block(&mut self, b: Block) -> Result<BlockNum> { panic!() }

    fn commit(self) -> Result<()> { panic!() }

    fn rollback(self) -> Result<()> { panic!() }
}
