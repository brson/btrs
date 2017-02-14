use std::mem;
use errors::*;
use wal_index::*;
use std::path::Path;
use std::convert::AsRef;
use std::marker::PhantomData;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::iter;
use fs2::FileExt;
use units::PageSize;
use byteorder::*;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::cell::RefCell;
use lock::*;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;

const MAGIC: u64 = 0x11a8b23d4760cdb4;

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

pub type PageNum = u32;
pub type FrameNum = u32;

pub struct Wal {
    file: Rc<RefCell<File>>,
    index: WalIndex,
    page_size: PageSize,
    epoch: u64,
    frame_map: FrameMap,
}

struct FrameMap {
    num_frames: u32,
    pages: BTreeMap<PageNum, FrameNum>,
}

pub struct ReadWal<'a> {
    wal: &'a mut Wal,
    lock: ReadLock,
}

pub struct WriteWal<'a> {
    wal: &'a mut Wal,
    lock: WriteLock,
}

pub struct Checkpoint<'a> {
    wal: &'a mut Wal,
    clock: CheckpointLock,
    wlock: WriteLock,
}

const HEADER_SIZE: u32 = 512;
const FRAME_HEADER_SIZE: u32 = 16;

#[derive(Debug, Eq, PartialEq)]
struct Header {
    magic: u64,
    page_size: PageSize,
    epoch: u64,
}


struct ReadLock(ShLock);
struct WriteLock(ExLock);
struct CheckpointLock(ExLock);
trait ReadOrWriteLock { }
impl ReadOrWriteLock for ReadLock { }
impl ReadOrWriteLock for WriteLock { }

impl Wal {
    pub fn new<P: AsRef<Path>>(p: P, page_size: PageSize) -> Result<Wal> {

        let index = WalIndex::new(p.as_ref())?;

        let file = OpenOptions::new()
            .read(true).write(true).create(true)
            .open(p.as_ref().with_extension("wal"))?;

        let mut wal = Wal {
            file: Rc::new(RefCell::new(file)),
            index: index,
            page_size: page_size,
            epoch: 0,
            frame_map: FrameMap {
                num_frames: 0,
                pages: BTreeMap::new(),
            },
        };

        wal.init(page_size)?;

        Ok(wal)
    }

    fn init(&mut self, page_size: PageSize) -> Result<()> {
        let wlock = self.write_lock()?;
        let clock = self.checkpoint_lock()?;
        let header = self.read_header(&wlock)?;
        let mut write = true;
        let header = if let Some(mut h) = header {
            let header_ok = h.magic == MAGIC || h.page_size == page_size;
            if !header_ok {
                Header {
                    magic: MAGIC,
                    page_size: page_size,
                    epoch: 0,
                }
            } else {
                write = false;
                h
            }
        } else {
            Header {
                magic: MAGIC,
                page_size: page_size,
                epoch: 0,
            }
        };

        self.epoch = header.epoch;

        if write {
            self.write_header(header, &clock)?;
        }

        Ok(())
    }

    pub fn begin_read(&mut self) -> Result<ReadWal> {
        Ok(ReadWal::new(self)?)
    }

    pub fn begin_write(&mut self) -> Result<WriteWal> {
        Ok(WriteWal::new(self)?)
    }

    pub fn begin_checkpoint(&mut self) -> Result<Checkpoint> {
        Ok(Checkpoint::new(self)?)
    }

    fn read_header(&mut self, lock: &ReadOrWriteLock) -> Result<Option<Header>> {
        let mut file = self.file.borrow_mut();

        if file.metadata()?.len() == 0 {
            return Ok(None);
        }

        file.seek(SeekFrom::Start(0))?;

        let mut rdr = BufReader::new(&mut *file);
        
        let magic = rdr.read_u64::<LittleEndian>()?;
        let page_size = rdr.read_u32::<LittleEndian>()?;
        let epoch = rdr.read_u64::<LittleEndian>()?;

        let header = Header {
            magic: magic,
            page_size: PageSize::new(page_size),
            epoch: epoch,
        };

        Ok(Some(header))
    }

    fn write_header(&mut self, h: Header, lock: &CheckpointLock) -> Result<()> {
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start(0))?;
        let mut wtr = BufWriter::new(&mut *file);
        wtr.write_u64::<LittleEndian>(h.magic)?;
        wtr.write_u32::<LittleEndian>(h.page_size.to_u32())?;
        wtr.write_u64::<LittleEndian>(h.epoch)?;

        Ok(())
    }

    fn frame_size(&self) -> u32 {
        self.page_size.to_u32() + FRAME_HEADER_SIZE
    }

    fn frame_offset(&self, fr: FrameNum) -> u64 {
        HEADER_SIZE as u64 + fr as u64 * self.frame_size() as u64
    }

    fn seek_frame(&self, fr: FrameNum) -> Result<()> {
        self.file.borrow_mut().seek(SeekFrom::Start(self.frame_offset(fr)))?;

        Ok(())
    }

    fn read_lock(&self) -> Result<ReadLock> {
        Ok(ReadLock(ShLock::new(self.file.clone())?))
    }

    fn write_lock(&self) -> Result<WriteLock> {
        Ok(WriteLock(self.index.write_lock()?))
    }

    fn checkpoint_lock(&self) -> Result<CheckpointLock> {
        Ok(CheckpointLock(ExLock::new(self.file.clone())?))
    }

    fn read_page(&self, bn: PageNum, lock: &ReadOrWriteLock) -> Result<Option<Page>>
    {
        if let Some(frame_num) = self.frame_map.pages.get(&bn).cloned() {
            let mut page = Page::new(self.page_size);

            // FIXME: Double seek
            self.seek_frame(frame_num)?;
            let mut file = self.file.borrow_mut();
            let u8_sz = mem::size_of::<u32>() as i64;
            let u64_sz = mem::size_of::<u64>() as i64;
            let header_sz = u8_sz * 2 + u64_sz;
            file.seek(SeekFrom::Current(header_sz))?;
            file.read_exact(page.buf_mut())?;

            Ok(Some(page))
        } else {
            Ok(None)
        }
    }

    fn update_frame_map(&mut self, lock: &ReadOrWriteLock) -> Result<()> {
        let file_len = self.file.borrow().metadata()?.len();

        let header = self.read_header(lock)?.expect("bad header");
        if header.epoch != self.epoch {
            println!("new epoch: {}", header.epoch);
            self.frame_map.pages.clear();
            self.frame_map.num_frames = 0;
            self.epoch = header.epoch;
        }

        let mut uncommitted = BTreeMap::new();
        
        for frame in self.frame_map.num_frames.. {
            self.seek_frame(frame)?;

            let r = || -> Result<(PageNum, u32)> {
                let err = || Err(IoError::from(IoErrorKind::UnexpectedEof).into());
                let mut file = self.file.borrow_mut();
                let mut rdr = BufReader::new(&mut *file);
                let page_num = rdr.read_u32::<LittleEndian>()?;
                let commit_flag = rdr.read_u32::<LittleEndian>()?;
                let epoch = rdr.read_u64::<LittleEndian>()?;

                if epoch != self.epoch {
                    return err();
                }

                // Is there actually space allocated for this frame?
                let next_frame_offset = self.frame_offset(frame + 1);
                if next_frame_offset > file_len {
                    return err();
                }
                Ok((page_num, commit_flag))
            }();

            match r {
                Err(Error(ErrorKind::Io(ref e), ..))
                    if e.kind() == IoErrorKind::UnexpectedEof =>
                {
                    // No more frames
                    return Ok(());
                }
                Err(e) => return Err(e),
                Ok((page_num, commit_flag)) => {
                    uncommitted.insert(page_num, frame);
                    // If this is a commit frame then update the frame
                    // map.
                    if commit_flag != 0 {
                        let new_pages = mem::replace(&mut uncommitted, BTreeMap::new());
                        self.frame_map.pages.extend(new_pages.into_iter());
                        self.frame_map.num_frames = frame + 1;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn dump(&self) {
        println!("----");
        println!("epoch: {}", self.epoch);
        println!("frames: {}", self.frame_map.num_frames);
        for (page, frame)  in &self.frame_map.pages {
            println!("page {}: frame {}", page, frame);
        }
    }
}

impl<'a> ReadWal<'a> {
    fn new(wal: &'a mut Wal) -> Result<ReadWal<'a>> {
        let lock = wal.read_lock()?;
        wal.update_frame_map(&lock)?;
        Ok(ReadWal {
            wal: wal,
            lock: lock,
        })
    }

    pub fn read_page(&self, i: PageNum) -> Result<Option<Page>> {
        self.wal.read_page(i, &self.lock)
    }

    pub fn begin_write(self) -> Result<WriteWal<'a>> {
        // Drop the read lock before taking the write lock.
        // update_frame_map will deal with any log changes between the
        // two. FIXME: this is too tricky!
        let wal = self.wal;
        { let _ = self.lock; }
        Ok(WriteWal::new(wal)?)
    }
}

impl<'a> WriteWal<'a> {
    fn new(wal: &'a mut Wal) -> Result<WriteWal<'a>> {
        let lock = wal.write_lock()?;
        wal.update_frame_map(&lock)?;
        Ok(WriteWal {
            wal: wal,
            lock: lock,
        })
    }

    pub fn read_page(&self, i: PageNum) -> Result<Option<Page>> {
        self.wal.read_page(i, &self.lock)
    }

    /// # Note
    ///
    /// This can be used to write page numbers beyond the end of the
    /// page store. Wabl will know to extend the page store during
    /// checkpointing.
    pub fn write_page(&mut self, i: PageNum, b: Page) -> Result<()> {
        let frame_num = self.wal.frame_map.num_frames;
        self.write_frame(frame_num, i, b)?;
        self.wal.frame_map.num_frames += 1;
        self.wal.frame_map.pages.insert(i, frame_num);

        Ok(())
    }

    fn write_frame(&mut self, frame_num: FrameNum,
                   page_num: PageNum, b: Page) -> Result<()>
    {
        assert!(b.0.len() as u32 == self.wal.page_size.to_u32());
        self.wal.seek_frame(frame_num)?;
        let mut file = self.wal.file.borrow_mut();
        let mut wtr = BufWriter::new(&mut *file);
        wtr.write_u32::<LittleEndian>(page_num)?;
        wtr.write_u32::<LittleEndian>(0)?;
        wtr.write_u64::<LittleEndian>(self.wal.epoch)?;
        wtr.write_all(&b.0)?;
        wtr.flush()?; // FIXME don't flush. Need a better BufWriter?

        Ok(())
    }

    pub fn commit(&mut self) -> Result<()> {
        if self.wal.frame_map.num_frames == 0 {
            return Ok(());
        }

        // Set the commit flag on last written page
        // FIXME: 2 seeks is 2 too many
        let frame = self.wal.frame_map.num_frames - 1;
        self.wal.seek_frame(frame)?;
        let mut file = self.wal.file.borrow_mut();
        file.seek(SeekFrom::Current(mem::size_of::<u32>() as i64))?;
        file.write_u32::<LittleEndian>(1)?;

        Ok(())
    }
}

use std::collections::btree_map;
type Pages<'a> = btree_map::Keys<'a, PageNum, FrameNum>;

impl<'a> Checkpoint<'a> {
    fn new(wal: &'a mut Wal) -> Result<Checkpoint<'a>> {
        // NB field order / unlocking order
        let wlock = wal.write_lock()?;
        let clock = wal.checkpoint_lock()?;
        wal.update_frame_map(&wlock)?;
        wal.file.borrow().sync_data()?;
        Ok(Checkpoint {
            wal: wal,
            clock: clock,
            wlock: wlock,
        })
    }

    pub fn pages(&self) -> Pages {
        self.wal.frame_map.pages.keys()
    }

    pub fn read_page(&self, i: PageNum) -> Result<Option<Page>> {
        self.wal.read_page(i, &self.wlock)
    }

    pub fn next_epoch(self) -> Result<()> {
        // Update the epoch in the header. Don't update our own
        // header state. That will happen like any other reader next
        // time we do a read.
        let header = Header {
            magic: MAGIC,
            page_size: self.wal.page_size,
            epoch: self.wal.epoch + 1,
        };
        self.wal.write_header(header, &self.clock)?;
        
        Ok(())
    }
}
