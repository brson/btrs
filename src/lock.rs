use errors::*;
use fs2::FileExt;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;

pub struct ExLock(Rc<RefCell<File>>);

pub struct ShLock(Rc<RefCell<File>>);

impl ExLock {
    pub fn new(file: Rc<RefCell<File>>) -> Result<ExLock> {
        file.borrow().lock_exclusive()?;
        Ok(ExLock(file))
    }
}

impl Drop for ExLock {
    fn drop(&mut self) {
        self.0.borrow().unlock().expect("unlock");
    }
}

impl ShLock {
    pub fn new(file: Rc<RefCell<File>>) -> Result<ShLock> {
        file.borrow().lock_shared()?;
        Ok(ShLock(file))
    }
}

impl Drop for ShLock {
    fn drop(&mut self) {
        self.0.borrow().unlock().expect("unlock");
    }
}
