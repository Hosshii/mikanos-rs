#![cfg_attr(not(test), no_std)]

pub mod log;
pub mod map;
pub mod ring_buf;
pub mod zeroed;

pub use zeroed::Zeroed;
