use crate::common::config::{BUSTUB_PAGE_SIZE, DEFAULT_DB_IO_SIZE, INITIAL_PAGE_CAPACITY, PageId};
use crate::common::util::read_exact_at;
use crate::error::{BustubError, Result};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::sync::Mutex;

pub type PageBuf = [u8; BUSTUB_PAGE_SIZE as usize];

#[derive(Debug)]
pub struct DiskManager {
    pub db_file: File,
    pub log_file: File,
    pub state: Mutex<ManagerState>,
}

#[derive(Debug)]
pub struct ManagerState {
    pub pages: HashMap<PageId, u64>,
    pub free_slots: Vec<u64>,
    pub db_file_len: u64,
    pub allocated_capacity: u64,
    pub page_capacity: u32,
}

impl DiskManager {
    pub fn new(db_file_path: impl AsRef<Path>) -> Result<Self> {
        let db_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&db_file_path)?;

        let db_file_len = db_file.metadata()?.len();
        if !db_file_len.is_multiple_of(BUSTUB_PAGE_SIZE as u64) {
            return Err(BustubError::NotPageAligned {
                file_len: db_file_len,
                page_size: BUSTUB_PAGE_SIZE as usize,
            });
        }

        let log_file_path = db_file_path.as_ref().with_extension("log");
        let log_file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&log_file_path)?;

        let initial_capacity = if db_file_len == 0 {
            let bytes_to_reserve = INITIAL_PAGE_CAPACITY as u64 * BUSTUB_PAGE_SIZE as u64;
            db_file.set_len(bytes_to_reserve)?;
            bytes_to_reserve
        } else {
            db_file_len
        };

        Ok(Self {
            db_file,
            log_file,
            state: Mutex::new(ManagerState {
                pages: HashMap::new(),
                free_slots: Vec::new(),
                db_file_len: 0,
                allocated_capacity: initial_capacity,
                page_capacity: DEFAULT_DB_IO_SIZE,
            }),
        })
    }

    pub fn write_page(&self, page_id: PageId, buf: &PageBuf) -> Result<()> {
        let target_offset: u64;
        {
            let mut state: std::sync::MutexGuard<'_, ManagerState> = self.state.lock().unwrap();

            target_offset = match state.pages.get(&page_id) {
                Some(&offset) => offset,
                None => {
                    let offset = self.allocate_page(&mut state)?;
                    state.pages.insert(page_id, offset);
                    offset
                }
            }
        }

        self.db_file.write_at(buf, target_offset)?;

        Ok(())
    }

    pub fn read_page(
        &self,
        page_id: PageId,
        buf: &mut [u8; BUSTUB_PAGE_SIZE as usize],
    ) -> Result<()> {
        let offset = {
            let mut state = self.state.lock().unwrap();
            match state.pages.get(&page_id).copied() {
                Some(offset) => offset,
                None => {
                    let offset = self.allocate_page(&mut state)?;
                    state.pages.insert(page_id, offset);
                    offset
                }
            }
        };

        let file_size = self.db_file.metadata()?.len();
        if offset > file_size {
            return Err(BustubError::PageReadPastEof {
                page_id: page_id.to_index(),
                offset: offset as u32,
            });
        }

        let filled = read_exact_at(&self.db_file, buf, offset)? as usize;
        buf[filled..].fill(0);

        Ok(())
    }

    pub fn read_log(&self, buf: &mut [u8], offset: u64) -> Result<bool> {
        let log_file_size = self.log_file.metadata()?.len();
        if offset > log_file_size {
            return Ok(false);
        }

        let read = read_exact_at(&self.log_file, buf, offset)? as usize;
        buf[read..].fill(0);
        Ok(true)
    }

    pub fn write_log(&self, buf: &[u8]) -> Result<()> {
        (&self.log_file).write_all(buf)?;
        Ok(())
    }

    pub fn allocate_page(&self, state: &mut ManagerState) -> Result<u64> {
        let new_offset = if let Some(reused) = state.free_slots.pop() {
            reused
        } else {
            let current_len = state.db_file_len;
            if current_len + BUSTUB_PAGE_SIZE as u64 > state.allocated_capacity {
                let new_capacity = state.allocated_capacity * 2;
                self.db_file.set_len(new_capacity)?;
                state.allocated_capacity = new_capacity;
            }

            state.db_file_len += BUSTUB_PAGE_SIZE as u64;
            current_len
        };

        Ok(new_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const DB_FILE_NAME: &str = "test.bustub";
    pub const DB_LOG_NAME: &str = "test.log";

    #[test]
    fn read_write_page_test() {
        let mut buf: [u8; BUSTUB_PAGE_SIZE as usize] = [0; BUSTUB_PAGE_SIZE as usize];
        let mut data: [u8; BUSTUB_PAGE_SIZE as usize] = [0; BUSTUB_PAGE_SIZE as usize];

        let page_id = PageId::new(0);
        let dm = DiskManager::new(DB_FILE_NAME).unwrap();

        let s = "A test string";
        data[..s.len()].copy_from_slice(&s.as_bytes());

        dm.read_page(page_id, &mut buf)
            .expect("failed to read empty page"); // tolerate empty read

        dm.write_page(page_id, &data).unwrap();
        dm.read_page(page_id, &mut buf).unwrap();
        assert_eq!(buf, data);

        buf.fill(0);

        let page_id = PageId::new(5);

        dm.write_page(page_id, &data).unwrap();
        dm.read_page(page_id, &mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn read_write_log_test() {
        let mut buf: [u8; 16] = [0; 16];
        let mut data: [u8; 16] = [0; 16];

        let dm = DiskManager::new(DB_FILE_NAME).unwrap();

        let s = "A test string";
        data[..s.len()].copy_from_slice(&s.as_bytes());

        dm.read_log(&mut buf, 0).expect("failed to read empty page"); // tolerate empty read
        dm.write_log(&data).unwrap();
        dm.read_log(&mut buf, 0).unwrap();

        assert_eq!(buf, data);
    }
}
