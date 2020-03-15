extern crate space_domain;
extern crate space_macros;

use space_macros::*;

use std::os::raw::c_char;
use std::ffi::CStr;

pub struct Context {

}

impl Context {
    pub fn new(args: &str) -> Context {
        info!("creating context with arguments {:?}", args);

        Context {

        }
    }
}

#[no_mangle]
pub extern "C" fn init(value: *const c_char) -> *mut Context {
    let c_str = unsafe {
        assert!(!value.is_null());
        CStr::from_ptr(value)
    };

    let value = c_str.to_str().unwrap();

    let mut context = Context::new(value);
    Box::into_raw(Box::new(context))
}

#[no_mangle]
pub extern fn close(ctx_ptr: *mut Context) {
    if ctx_ptr.is_null() { return }
    let ctx = unsafe { Box::from_raw(ctx_ptr); };
    info!("closing context {:?}", ctx);
}

