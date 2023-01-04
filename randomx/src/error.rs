use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("FailedToInitializeVM")]
    FailedToInitializeVM,
    #[error("FailedToInitializeDataset")]
    FailedToInitializeDataset,

    #[error("ThreadError")]
    ThreadError,
    #[error("OutSizeLessThanHashLen {0}")]
    OutSizeLessThanHashLen(usize),
    #[error("DatasetAllocError")]
    DatasetAllocError,
    #[error("CacheAllocError")]
    CacheAllocError,
}
