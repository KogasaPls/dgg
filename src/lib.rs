#![allow(dead_code)]
#![feature(lazy_cell)]

#[macro_use]
pub mod common;
pub mod config;
pub mod dgg;

#[macro_use]
extern crate log;
extern crate core;
#[cfg(test)]
extern crate ctor;
extern crate dotenv;

#[cfg(test)]
#[ctor::ctor]
fn init() {
    dotenv::dotenv().ok();
    pretty_env_logger::formatted_timed_builder()
        .parse_env("RUST_LOG")
        .init();

    debug!("Initialized test environment");
}
