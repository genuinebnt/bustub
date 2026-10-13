use thiserror::Error;

pub type Result<T, E = BustubError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum BustubError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DB File Size ({file_len} bytes) is corrupted: not a multiple of page size ({page_size}) bytes")]
    NotPageAligned { file_len: u64, page_size: usize }
}