#[unsafe(no_mangle)]
pub extern "C" fn protocol_init() -> i32 {
    42
}

#[unsafe(no_mangle)]
pub extern "C" fn protocol_send(data: *const u8, len: usize) -> i32 {
    let slice = unsafe {
        std::slice::from_raw_parts(data, len)
    };
    println!("Received: {:?}", slice);
    0
}