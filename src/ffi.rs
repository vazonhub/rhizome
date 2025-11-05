#[cfg(feature = "ffi")]
use std::os::raw::{c_int, c_uchar};
use crate::{protocol_init, protocol_send, protocol_receive};

#[no_mangle]
pub unsafe extern "C" fn rhizome_init() -> c_int {
    protocol_init()
}

#[no_mangle]
pub unsafe extern "C" fn rhizome_send(data: *const c_uchar, len: usize) -> c_int {
    if data.is_null() { return -1; }
    let slice = std::slice::from_raw_parts(data, len);
    protocol_send(slice)
}

#[no_mangle]
pub unsafe extern "C" fn rhizome_receive(data: *const c_uchar, len: usize) -> c_int {
    if data.is_null() { return -1; }
    let slice = std::slice::from_raw_parts(data, len);
    protocol_receive(slice)
}