use std::fs::{File, OpenOptions};
use std::os::unix::fs::FileExt;
use std::sync::Mutex;
use std::path::Path;
use crate::common::config::{BUSTUB_PAGE_SIZE, DEFAULT_DB_IO_SIZE, INITIAL_PAGE_CAPACITY, PageId};
use crate::error::{self, Result};
use std::collections::HashMap;

pub type PageBuf = [u8; BUSTUB_PAGE_SIZE as usize];

#[derive(Debug)]
pub struct DiskManager {
    pub db_file: File,
    pub state: Mutex<ManagerState>,
    pub page_capacity: u32,
}

#[derive(Debug)]
pub struct ManagerState {
    pub pages: HashMap<PageId, u64>,
    pub free_slots: Vec<u64>,
    pub db_file_len: u64,
    pub allocated_capacity: u64
}

impl DiskManager {
    pub fn new(db_file_path: impl AsRef<Path>) -> Result<Self> {
        let db_file = OpenOptions::new().read(true).write(true).create(true).truncate(true).open(&db_file_path)?;
        let db_file_len = db_file.metadata()?.len();

        if !db_file_len.is_multiple_of(BUSTUB_PAGE_SIZE as u64) {
            return Err(error::BustubError::NotPageAligned { file_len: db_file_len, page_size: BUSTUB_PAGE_SIZE as usize })
        }

        let initial_capacity = if db_file_len == 0 {
            let bytes_to_reserve = INITIAL_PAGE_CAPACITY as u64 * BUSTUB_PAGE_SIZE as u64;
            db_file.set_len(bytes_to_reserve)?;
            bytes_to_reserve
        } else {
            db_file_len
        };

        Ok(Self {
            db_file: db_file,
            state: Mutex::new(ManagerState { pages: HashMap::new(), free_slots: Vec::new(), db_file_len: 0, allocated_capacity: initial_capacity }),
            page_capacity: DEFAULT_DB_IO_SIZE
        })
    }

    pub fn write_page(&self, page_id: PageId, buf: &PageBuf) -> Result<()> {
        let target_offset: u64;

        {
            let mut guard = self.state.lock().unwrap();
            
            target_offset = match guard.pages.get(&page_id) {
                Some(&offset) => offset,
                None => {
                    let new_offset = if let Some(reused) = guard.free_slots.pop() {
                        reused
                    } else {
                        let current_len = guard.db_file_len;
                        if current_len + BUSTUB_PAGE_SIZE as u64 > guard.allocated_capacity {
                            let new_capacity = guard.allocated_capacity * 2;
                            self.db_file.set_len(new_capacity)?;
                            guard.allocated_capacity = new_capacity;
                        }

                        guard.db_file_len += BUSTUB_PAGE_SIZE as u64;
                        current_len
                    };

                    guard.pages.insert(page_id, new_offset);
                    new_offset
                }
            }
        }

        self.db_file.write_at(buf, target_offset)?;

        Ok(())
    }
}