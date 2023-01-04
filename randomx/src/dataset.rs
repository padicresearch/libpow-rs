use crate::bindings::{
    randomx_alloc_cache, randomx_alloc_dataset, randomx_cache, randomx_dataset,
    randomx_dataset_item_count, randomx_init_cache, randomx_init_dataset, randomx_release_cache,
    randomx_release_dataset,
};
use crate::error::Error;
use crate::flag::RandomxFlags;
use std::ffi::{c_ulong};
use std::sync::Arc;

pub struct RandomxDataset {
    pub(crate) dataset: *mut randomx_dataset,
}

impl RandomxDataset {
    pub fn new(flags: RandomxFlags, key: &[u8]) -> Result<Self, Error> {
        unsafe {
            let cache = RandomxCache::new(flags, key)?;
            let mut dataset = RandomxDataset {
                dataset: randomx_alloc_dataset(flags.bits()),
            };
            let count = randomx_dataset_item_count();
            let num_cpus = num_cpus::get() as c_ulong;
            if num_cpus <= 1 {
                randomx_init_dataset(dataset.dataset, cache.cache, 0, count);
            } else {
                let arc_cache = Arc::new(cache);
                let arc_dataset = Arc::new(dataset);

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
                    handles.push(std::thread::spawn(move || {
                        randomx_init_dataset(dataset.dataset, cache.cache, start_index, temp_size);
                    }));
                    start += temp_size;
                }

                for handle in handles {
                    handle.join().map_err(|_| Error::ThreadError)?;
                }

                dataset = match Arc::try_unwrap(arc_dataset) {
                    Ok(dataset) => dataset,
                    Err(_) => return Err(Error::DatasetAllocError),
                };
            }

            Ok(dataset)
        }
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
        let cache = unsafe { randomx_alloc_cache(flags.bits()) };

        if cache.is_null() {
            return Err(Error::CacheAllocError);
        }

        unsafe {
            randomx_init_cache(cache, key.as_ptr() as *const std::ffi::c_void, key.len());
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
