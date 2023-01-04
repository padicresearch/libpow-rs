extern crate core;

use crate::dataset::{RandomxCache, RandomxDataset};
use crate::error::Error;
use crate::flag::RandomxFlags;
use std::ffi::c_void;

pub(crate) mod bindings;
mod dataset;
mod error;
pub mod flag;

pub const RANDOMX_HASH_SIZE: usize = bindings::RANDOMX_HASH_SIZE as usize;

pub struct RandomX<'a> {
    vm: &'a mut bindings::randomx_vm,
}

impl<'a> RandomX<'a> {
    pub fn new(flags: RandomxFlags, cache: &mut RandomxCache<'a>) -> Result<Self, Error> {
        let vm = unsafe {
            let flags = flags.bits();
            let vm = bindings::randomx_create_vm(flags, cache.cache, std::ptr::null_mut()).as_mut();
            vm
        };
        let Some(vm) = vm else {
            return Err(Error::FailedToInitializeVM);
        };
        Ok(Self { vm })
    }

    pub fn new_with_key(flags: RandomxFlags, key: &[u8]) -> Result<Self, Error> {
        let cache = RandomxCache::new(flags, key)?;
        let vm = unsafe {
            let flags = flags.bits();
            let vm = bindings::randomx_create_vm(flags, cache.cache, std::ptr::null_mut()).as_mut();
            vm
        };
        let Some(vm) = vm else {
            return Err(Error::FailedToInitializeVM);
        };
        Ok(Self { vm })
    }

    pub fn new_fast(flags: RandomxFlags, dataset: &mut RandomxDataset<'a>) -> Result<Self, Error> {
        let vm = unsafe {
            let flags = flags.bits();
            let vm =
                bindings::randomx_create_vm(flags, std::ptr::null_mut(), dataset.dataset).as_mut();
            vm
        };
        let Some(vm) = vm else {
            return Err(Error::FailedToInitializeVM);
        };
        Ok(Self { vm })
    }

    pub fn calculate_hash<I: AsRef<[u8]>>(
        &mut self,
        input: I,
        out: &mut [u8],
    ) -> Result<(), Error> {
        if out.len() < RANDOMX_HASH_SIZE {
            return Err(Error::OutSizeLessThanHashLen(RANDOMX_HASH_SIZE));
        }
        self._calculate_hash(input, out);
        Ok(())
    }

    pub fn calculate_hash_to_vec<I: AsRef<[u8]>>(&mut self, input: I) -> Vec<u8> {
        let mut out = [0_u8; RANDOMX_HASH_SIZE];
        self._calculate_hash(input, &mut out);
        out.into()
    }

    fn _calculate_hash<I: AsRef<[u8]>>(&mut self, input: I, out: &mut [u8]) {
        unsafe {
            bindings::randomx_calculate_hash(
                self.vm,
                input.as_ref().as_ptr() as *const c_void,
                input.as_ref().len(),
                out.as_mut_ptr() as *mut c_void,
            );
        }
    }
}

impl<'a> Drop for RandomX<'a> {
    fn drop(&mut self) {
        unsafe {
            bindings::randomx_destroy_vm(self.vm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let flags = RandomxFlags::default();
        let key = b"RandomX example key\0";
        let input = b"RandomX example input\0";
        let mut out = [0_u8; RANDOMX_HASH_SIZE];

        let mut cache = RandomxCache::new(flags, key).unwrap();

        let mut rx = RandomX::new(flags, &mut cache).unwrap();
        rx.calculate_hash(input, &mut out).unwrap();
        let expected = [
            138, 72, 229, 249, 219, 69, 171, 121, 217, 8, 5, 116, 196, 216, 25, 84, 254, 106, 198,
            56, 66, 33, 74, 255, 115, 194, 68, 178, 99, 48, 183, 201,
        ];

        assert_eq!(expected, out);

        println!("{}", hex::encode(out));
    }

    #[test]
    fn basic_fast() {
        let flags = RandomxFlags::default() | RandomxFlags::FULLMEM;
        let key = b"RandomX example key\0";
        let input = b"RandomX example input\0";
        let mut out = [0_u8; RANDOMX_HASH_SIZE];

        let mut dataset = RandomxDataset::new(flags, key).unwrap();

        let mut rx = RandomX::new_fast(flags, &mut dataset).unwrap();
        rx.calculate_hash(input, &mut out).unwrap();
        let expected = [
            138, 72, 229, 249, 219, 69, 171, 121, 217, 8, 5, 116, 196, 216, 25, 84, 254, 106, 198,
            56, 66, 33, 74, 255, 115, 194, 68, 178, 99, 48, 183, 201,
        ];
        assert_eq!(expected, out);
    }
}
