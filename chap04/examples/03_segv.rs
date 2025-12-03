fn main() {
    unsafe {
        let p: *mut i32 = std::ptr::null_mut();
        *p = 0;
    }
}