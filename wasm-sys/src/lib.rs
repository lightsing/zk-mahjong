#![feature(lazy_cell)]
#[macro_use] extern crate log;

use wasm_bindgen::prelude::*;

pub mod keys;
pub mod tiles;
pub(crate) mod utils;

#[wasm_bindgen(js_name = "zkMahjongInit")]
pub fn init() {
    #[cfg(feature = "debuggable")]
    {
        static LOG_INIT: std::sync::Once = std::sync::Once::new();
        console_error_panic_hook::set_once();
        LOG_INIT.call_once(|| {
            wasm_logger::init(wasm_logger::Config::default());
        });
        info!("zkMahjong Initialized");
    }
}