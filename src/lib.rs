mod error_code;
pub mod instruction;
mod payment;
mod satomic_swap;
#[cfg(test)]
mod tests;
mod utils;

pub const STORAGE_SPACE_ALLOCATED: u64 = 41;
