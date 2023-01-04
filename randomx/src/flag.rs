use crate::bindings::{
    randomx_flags_RANDOMX_FLAG_ARGON2, randomx_flags_RANDOMX_FLAG_ARGON2_AVX2,
    randomx_flags_RANDOMX_FLAG_ARGON2_SSSE3, randomx_flags_RANDOMX_FLAG_DEFAULT,
    randomx_flags_RANDOMX_FLAG_FULL_MEM, randomx_flags_RANDOMX_FLAG_HARD_AES,
    randomx_flags_RANDOMX_FLAG_JIT, randomx_flags_RANDOMX_FLAG_LARGE_PAGES,
    randomx_flags_RANDOMX_FLAG_SECURE,
};
use bitflags::bitflags;

bitflags! {
    /// Represents options that can be used when allocating the
    /// RandomX dataset or VM.
    #[derive(Copy,Clone)]
    pub struct RandomxFlags: u32 {
        /// Use defaults.
        const DEFAULT = randomx_flags_RANDOMX_FLAG_DEFAULT;

        /// Allocate memory in large pages.
        const LARGEPAGES = randomx_flags_RANDOMX_FLAG_LARGE_PAGES;

        /// The RandomX VM will use hardware accelerated AES.
        const HARDAES = randomx_flags_RANDOMX_FLAG_HARD_AES;

        /// The RandomX VM will use the full dataset.
        const FULLMEM = randomx_flags_RANDOMX_FLAG_FULL_MEM;

        /// The RandomX VM will use a JIT compiler.
        const JIT = randomx_flags_RANDOMX_FLAG_JIT;

        /// Make sure that JIT pages are never writable and executable
        /// at the same time.
        const SECURE = randomx_flags_RANDOMX_FLAG_SECURE;

        /// Use the SSSE3 extension to speed up Argon2 operations.
        const ARGON2_SSSE3 = randomx_flags_RANDOMX_FLAG_ARGON2_SSSE3;

        /// Use the AVX2 extension to speed up Argon2 operations.
        const ARGON2_AVX2 = randomx_flags_RANDOMX_FLAG_ARGON2_AVX2;

        /// Do not use SSSE3 or AVX2 extensions.
        const ARGON2 = randomx_flags_RANDOMX_FLAG_ARGON2;
    }
}

impl Default for RandomxFlags {
    fn default() -> Self {
        RandomxFlags::DEFAULT
    }
}
