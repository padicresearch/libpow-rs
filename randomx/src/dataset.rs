use crate::bindings::{
    randomx_alloc_cache, randomx_alloc_dataset, randomx_cache, randomx_dataset,
    randomx_dataset_item_count, randomx_get_dataset_memory, randomx_init_cache,
    randomx_init_dataset, randomx_release_cache, randomx_release_dataset,
    RANDOMX_DATASET_ITEM_SIZE,
};
use crate::error::Error;
use crate::flag::RandomxFlags;
use memmap2::Mmap;
use parking_lot::Mutex;
use std::ffi::{c_ulong, c_void};
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

pub struct RandomxDataset {
    pub(crate) dataset: *mut randomx_dataset,
}

impl RandomxDataset {
    pub fn new(flags: RandomxFlags, key: &[u8]) -> Result<Self, Error> {
        let cache = RandomxCache::new(flags, key)?;
        let mut dataset = RandomxDataset {
            dataset: unsafe {
                randomx_alloc_dataset(flags.bits())
                    .as_mut()
                    .ok_or(Error::DatasetAllocError)?
            },
        };
        let count = unsafe { randomx_dataset_item_count() };
        let num_cpus = num_cpus::get() as c_ulong;
        if num_cpus <= 1 {
            unsafe {
                randomx_init_dataset(dataset.dataset, cache.cache, 0, count);
            }
        } else {
            let arc_cache = Arc::new(Mutex::new(cache));
            let arc_dataset = Arc::new(Mutex::new(dataset));

            let size = count / num_cpus;
            let last = count % num_cpus;
            let mut start = 0;
            let mut handles = Vec::new();
            for i in 0..num_cpus {
                let cache = arc_cache.clone();
                let dataset = arc_dataset.clone();
                let mut temp_size = size;
                if i == num_cpus - 1 {
                    temp_size += last;
                }
                let start_index = start;
                handles.push(std::thread::spawn(move || unsafe {
                    randomx_init_dataset(
                        (*(dataset.data_ptr())).dataset,
                        (*(cache.data_ptr())).cache,
                        start_index,
                        temp_size,
                    );
                }));
                start += temp_size;
            }

            for handle in handles {
                handle.join().map_err(|_| Error::ThreadError)?;
            }

            dataset = match Arc::try_unwrap(arc_dataset) {
                Ok(dataset) => dataset.into_inner(),
                Err(_) => return Err(Error::DatasetAllocError),
            };
        }

        Ok(dataset)
    }

    pub fn as_slice(&mut self) -> &[u8] {
        unsafe {
            let ptr = randomx_get_dataset_memory(self.dataset) as *const u8;
            let out = std::slice::from_raw_parts(
                ptr,
                (randomx_dataset_item_count() as usize * RANDOMX_DATASET_ITEM_SIZE as usize)
                    as usize,
            );
            out
        }
    }

    pub fn open<P: AsRef<Path>>(flags: RandomxFlags, path: P) -> Result<Self, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let mmap_file = unsafe { Mmap::map(&file)? };
        let dataset = unsafe { randomx_alloc_dataset(flags.bits()) };
        unsafe {
            libc::memcpy(
                randomx_get_dataset_memory(ptr),
                mmap_file.as_ptr() as *const _,
                randomx_dataset_item_count() as usize * RANDOMX_DATASET_ITEM_SIZE as usize,
            );
        }

        if dataset.is_null() {
            return Err(Error::DatasetAllocError);
        }
        Ok(Self { dataset })
    }
}

unsafe impl Send for RandomxDataset {}

unsafe impl Sync for RandomxDataset {}

impl Drop for RandomxDataset {
    fn drop(&mut self) {
        unsafe { randomx_release_dataset(self.dataset) }
    }
}

pub struct RandomxCache {
    pub(crate) cache: *mut randomx_cache,
}

impl RandomxCache {
    pub fn new(flags: RandomxFlags, key: &[u8]) -> Result<Self, Error> {
        let cache = unsafe {
            randomx_alloc_cache(flags.bits())
                .as_mut()
                .ok_or(Error::CacheAllocError)?
        };

        unsafe {
            randomx_init_cache(cache, key.as_ptr() as *const c_void, key.len());
        }

        Ok(RandomxCache { cache })
    }
}

impl Drop for RandomxCache {
    fn drop(&mut self) {
        unsafe { randomx_release_cache(self.cache) }
    }
}

unsafe impl Send for RandomxCache {}

unsafe impl Sync for RandomxCache {}
