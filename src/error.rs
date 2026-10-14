use thiserror::Error;

pub type Result<T, E = BustubError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum BustubError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "DB File Size ({file_len} bytes) is corrupted: not a multiple of page size ({page_size}) bytes"
    )]
    NotPageAligned { file_len: u64, page_size: usize },

    #[error("Page not found with id: ({page_id})")]
    PageNotFound { page_id: u32 },

    #[error("Failed to read page: ({page_id})")]
    PageReadError { page_id: u32 },

    #[error("Read page ({page_id}) past end of file offset: ({offset})")]
    PageReadPastEof { page_id: u32, offset: u32 },
}
