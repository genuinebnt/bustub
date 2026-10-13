use std::num::NonZeroU32;

pub const BUSTUB_PAGE_SIZE: u32 = 8192; // size of data page in bytes
pub const BUFFER_POOL_SIZE: u32 = 128; // size of buffer pool
pub const DEFAULT_DB_IO_SIZE: u32 = 16; // starting size of file on the disk
pub const LOG_BUFFER_SIZE: u32 = (BUFFER_POOL_SIZE + 1) * BUSTUB_PAGE_SIZE; // size of log buffer in bytes
pub const INITIAL_PAGE_CAPACITY: u32 = 1024;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(NonZeroU32);

impl PageId {
    pub fn new(num: u32) -> Self {
        let interval_val = NonZeroU32::new(num + 1).expect("Page id overflowed max u32 limit");
        Self(interval_val)
    }

    pub fn to_index(self) -> u32 {
        self.0.get() - 1
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameId(NonZeroU32);

impl FrameId {
    pub fn new(num: u32) -> Self {
        let interval_val = NonZeroU32::new(num + 1).expect("Frame id overflowed max u32 limit");
        Self(interval_val)
    }

    pub fn to_index(self) -> u32 {
        self.0.get() - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotOffset(NonZeroU32);

impl SlotOffset {
    pub fn new(num: u32) -> Self {
        let interval_val = NonZeroU32::new(num + 1).expect("Page id overflowed max u32 limit");
        Self(interval_val)
    }

    pub fn to_index(self) -> u32 {
        self.0.get() - 1
    }
}
