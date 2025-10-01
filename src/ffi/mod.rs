use once_cell::sync::OnceCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

const TLSN_SUCCESS: i32 = 0;
const TLSN_ERROR_INIT: i32 = -1;
const TLSN_ERROR_INVALID: i32 = -2;
const TLSN_ERROR_RUNTIME: i32 = -3;
const TLSN_ERROR_UNKNOWN: i32 = -99;

fn set_last_error(error: &str) {
    *LAST_ERROR.lock().unwrap() = Some(error.to_string());
}

unsafe fn c_str_to_rust_str(ptr: *const c_char) -> Result<&'static str, &'static str> {
    if ptr.is_null() {
        return Err("Null pointer");
    }

    unsafe {
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => Ok(s),
            Err(_) => Err("Invalid UTF-8 string"),
        }
    }
}

unsafe fn c_str_to_rust_option(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(ptr).to_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_init() -> i32 {
    match Runtime::new() {
        Ok(rt) => match RUNTIME.set(rt) {
            Ok(_) => TLSN_SUCCESS,
            Err(_) => {
                set_last_error("Runtime already initialized");
                TLSN_ERROR_INIT
            }
        },
        Err(e) => {
            set_last_error(&format!("Failed to create Tokio runtime: {}", e));
            TLSN_ERROR_RUNTIME
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_cleanup() {
    // Clear any stored error
    *LAST_ERROR.lock().unwrap() = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_prove(
    mode: i32,
    url: *const c_char,
    cookie: *const c_char,
    access_token: *const c_char,
    user_agent: *const c_char,
    provider_host: *const c_char,
    provider_port: u16,
    notary_host: *const c_char,
    notary_port: u16,
    notary_tls_enabled: bool,
    max_sent_data: usize,
    max_recv_data: usize,
) -> i32 {
    let rt = match RUNTIME.get() {
        Some(rt) => rt,
        None => {
            set_last_error("Library not initialized. Call tlsn_init() first.");
            return TLSN_ERROR_INIT;
        }
    };

    let url = match unsafe { c_str_to_rust_str(url) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid url string");
            return TLSN_ERROR_INVALID;
        }
    };

    let user_agent = match unsafe { c_str_to_rust_str(user_agent) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid user_agent string");
            return TLSN_ERROR_INVALID;
        }
    };

    let provider_host = match unsafe { c_str_to_rust_str(provider_host) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid provider_host string");
            return TLSN_ERROR_INVALID;
        }
    };

    let notary_host = match unsafe { c_str_to_rust_str(notary_host) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid notary_host string");
            return TLSN_ERROR_INVALID;
        }
    };

    let cookie = unsafe { c_str_to_rust_option(cookie) };
    let access_token = unsafe { c_str_to_rust_option(access_token) };

    let mode = match mode {
        0 => crate::domain::Mode::Prove,
        1 => crate::domain::Mode::Present,
        2 => crate::domain::Mode::ProveToPresent,
        _ => {
            set_last_error("Invalid mode value. Use 0=Prove, 1=Present, 2=ProveToPresent");
            return TLSN_ERROR_INVALID;
        }
    };

    match rt.block_on(crate::prove(
        &mode,
        url,
        cookie,
        access_token,
        user_agent,
        provider_host,
        provider_port,
        notary_host,
        notary_port,
        notary_tls_enabled,
        max_sent_data,
        max_recv_data,
    )) {
        Ok(_) => TLSN_SUCCESS,
        Err(e) => {
            set_last_error(&e.to_string());
            TLSN_ERROR_UNKNOWN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_verify(url: *const c_char, unauthed_bytes: *const c_char) -> i32 {
    let rt = match RUNTIME.get() {
        Some(rt) => rt,
        None => {
            set_last_error("Library not initialized. Call tlsn_init() first.");
            return TLSN_ERROR_INIT;
        }
    };

    let url = match unsafe { c_str_to_rust_str(url) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid url string");
            return TLSN_ERROR_INVALID;
        }
    };

    let unauthed_bytes = match unsafe { c_str_to_rust_str(unauthed_bytes) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid unauthed_bytes string");
            return TLSN_ERROR_INVALID;
        }
    };

    match rt.block_on(crate::verify(url, unauthed_bytes)) {
        Ok(_) => TLSN_SUCCESS,
        Err(e) => {
            set_last_error(&e.to_string());
            TLSN_ERROR_UNKNOWN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_get_last_error() -> *const c_char {
    let error_guard = LAST_ERROR.lock().unwrap();
    match error_guard.as_ref() {
        Some(error) => match CString::new(error.as_str()) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null(),
        },
        None => std::ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn tlsn_free_error_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
