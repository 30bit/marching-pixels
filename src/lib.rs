#![no_std]
#![cfg_attr(doc, feature(doc_cfg))]
#![allow(clippy::cast_possible_truncation)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod core;

#[cfg(feature = "alloc")]
mod algorithm;

#[cfg(feature = "alloc")]
#[cfg_attr(doc, doc(cfg(feature = "alloc")))]
pub use algorithm::{Algorithm, Args, IntoArgs};
