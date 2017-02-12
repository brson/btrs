use errors::*;
use std::path::Path;
use std::convert::AsRef;
use std::marker::PhantomData;

pub struct Block(Vec<u8>);

pub type BlockNum = u64;

pub struct Wal;

pub struct ReadWal<'a> {
    wal: &'a mut Wal,
}

pub struct WriteWal<'a> {
    wal: &'a mut Wal,
}

pub struct CheckpointIter;

impl Wal {
    pub fn new<P: AsRef<Path>>(p: &P) -> Result<Wal> { panic!() }

    pub fn begin_read(&mut self) -> Result<ReadWal> {
        Ok(ReadWal {
            wal: self
        })
    }
}

impl<'a> ReadWal<'a> {
    pub fn read_block(&mut self, i: BlockNum) -> Result<Option<Block>> { panic!() }

    pub fn begin_write(&mut self) -> Result<WriteWal> {
        Ok(WriteWal {
            wal: self.wal
        })
    }
}

impl<'a> WriteWal<'a> {
    pub fn read_block(&mut self, i: BlockNum) -> Result<Option<Block>> { panic!() }

    /// # Note
    ///
    /// This can be used to write block numbers beyond the end of the
    /// block store. Wabl will know to extend the block store during
    /// checkpointing.
    pub fn write_block(&mut self, i: BlockNum, b: Block) -> Result<()> { panic!() }

    pub fn commit(&mut self) -> Result<()> { panic!() }

    pub fn checkpoint(&mut self) -> Result<CheckpointIter> { panic!() }
}

