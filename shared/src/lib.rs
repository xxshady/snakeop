pub mod exports;
pub mod imports;
pub mod abi_stable_types;

pub const EXPORTS: &str = include_str!("exports.rs");
pub const IMPORTS: &str = include_str!("imports.rs");
