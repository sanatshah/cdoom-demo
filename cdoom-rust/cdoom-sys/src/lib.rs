//! Bindgen-generated Chocolate Doom C bindings.
//!
//! This crate intentionally stays close to the C surface: names, integer widths,
//! and signatures should match the headers so migration modules can preserve
//! demo-deterministic behavior while replacing one C module at a time.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
