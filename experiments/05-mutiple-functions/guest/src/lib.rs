extern "C" {
    fn from_host1();
    fn from_host2();
}

#[no_mangle]
pub extern "C" fn sum(x: i32) -> i32 {
    fpm_lib::from_host1();

    x + unsafe {
        from_host2();
        10
    }
}

mod fpm_lib {
    pub fn from_host1() {
        unsafe { super::from_host1() }
    }
}
