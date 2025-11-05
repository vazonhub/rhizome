#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::{protocol_init, protocol_send, protocol_receive};

#[wasm_bindgen]
pub fn init() -> i32 {
    protocol_init()
}

#[wasm_bindgen]
pub fn send(data: &[u8]) -> i32 {
    protocol_send(data)
}

#[wasm_bindgen]
pub fn receive(data: &[u8]) -> i32 {
    protocol_receive(data)
}