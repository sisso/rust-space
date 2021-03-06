extern crate space_domain;
extern crate space_macros;

use space_macros::*;

use space_domain::ffi::FFIApi;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::Duration;

pub struct Context<'a, 'b> {
    api: FFIApi<'a, 'b>,
}

#[repr(C)]
pub struct FfiContext {
    _priv: [u8; 0],
}

impl<'a, 'b> Context<'a, 'b> {
    fn from_ptr<'c>(ptr: *mut FfiContext) -> &'c mut Context<'a, 'b> {
        assert!(!ptr.is_null());

        unsafe { &mut *(ptr.cast()) }
    }

    pub fn new(args: &str) -> Context<'a, 'b> {
        debugf!("creating context with arguments {:?}", args);

        let mut api = FFIApi::new();
        api.new_game();

        Context { api }
    }
}

#[no_mangle]
pub extern "C" fn space_domain_init_context<'a, 'b>(value: *const c_char) -> *mut FfiContext {
    let c_str = unsafe {
        assert!(!value.is_null());
        CStr::from_ptr(value)
    };

    let value = c_str.to_str().unwrap();

    let mut context = Context::new(value);
    Box::into_raw(Box::new(context)).cast()
}

#[no_mangle]
pub extern "C" fn space_domain_close_context(ctx_ptr: *mut FfiContext) {
    if ctx_ptr.is_null() {
        return;
    }
    let ctx = unsafe { Box::from_raw(ctx_ptr) };
    debugf!("closing context");
    drop(ctx);
}

#[no_mangle]
pub extern "C" fn space_domain_set_data(
    ctx_ptr: *mut FfiContext,
    kind: u16,
    buffer: *mut u8,
    length: u32,
) -> u32 {
    let ctx = Context::from_ptr(ctx_ptr);
    let ref_data = unsafe { std::slice::from_raw_parts(buffer, length as usize) };
    let bytes = ref_data.to_vec();
    debugf!("set_data: {:?}", bytes);
    0
}

#[no_mangle]
pub extern "C" fn space_domain_get_data(
    ctx_ptr: *mut FfiContext,
    callback: extern "stdcall" fn(u16, *mut u8, u32),
) -> u32 {
    let ctx = Context::from_ptr(ctx_ptr);
    ctx.api.get_inputs(|mut bytes| {
        debugf!("get_data: {:?}", bytes);
        callback(0, bytes.as_mut_ptr(), bytes.len() as u32);
    });
    0
}

#[no_mangle]
pub extern "C" fn space_domain_run_tick(ctx_ptr: *mut FfiContext, delta_time: u32) -> u32 {
    debugf!("run_tick");
    let ctx = Context::from_ptr(ctx_ptr);
    ctx.api.update(Duration::from_millis(delta_time as u64));
    0
}
