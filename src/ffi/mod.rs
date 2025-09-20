use once_cell::sync::OnceCell;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

const ZKP2P_SUCCESS: i32 = 0;
const ZKP2P_ERROR_INIT: i32 = -1;
const ZKP2P_ERROR_INVALID: i32 = -2;
const ZKP2P_ERROR_RUNTIME: i32 = -3;
const ZKP2P_ERROR_UNKNOWN: i32 = -99;

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
pub extern "C" fn zkp2p_init() -> i32 {
    match Runtime::new() {
        Ok(rt) => match RUNTIME.set(rt) {
            Ok(_) => ZKP2P_SUCCESS,
            Err(_) => {
                set_last_error("Runtime already initialized");
                ZKP2P_ERROR_INIT
            }
        },
        Err(e) => {
            set_last_error(&format!("Failed to create Tokio runtime: {}", e));
            ZKP2P_ERROR_RUNTIME
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn zkp2p_cleanup() {
    // Clear any stored error
    *LAST_ERROR.lock().unwrap() = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn zkp2p_prove(
    mode: i32,
    provider: i32,
    transaction_id: *const c_char,
    profile_id: *const c_char,
    cookie: *const c_char,
    access_token: *const c_char,
) -> i32 {
    let rt = match RUNTIME.get() {
        Some(rt) => rt,
        None => {
            set_last_error("Library not initialized. Call zkp2p_init() first.");
            return ZKP2P_ERROR_INIT;
        }
    };

    let transaction_id = match unsafe { c_str_to_rust_str(transaction_id) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid transaction_id string");
            return ZKP2P_ERROR_INVALID;
        }
    };

    let profile_id = unsafe { c_str_to_rust_option(profile_id) };
    let cookie = unsafe { c_str_to_rust_option(cookie) };
    let access_token = unsafe { c_str_to_rust_option(access_token) };

    let mode = match mode {
        0 => crate::domain::Mode::Prove,
        1 => crate::domain::Mode::Present,
        2 => crate::domain::Mode::ProveToPresent,
        _ => {
            set_last_error("Invalid mode value. Use 0=Prove, 1=Present, 2=ProveToPresent");
            return ZKP2P_ERROR_INVALID;
        }
    };

    let provider = match provider {
        0 => crate::domain::Provider::Wise,
        1 => crate::domain::Provider::PayPal,
        _ => {
            set_last_error("Invalid provider value. Use 0=Wise, 1=PayPal");
            return ZKP2P_ERROR_INVALID;
        }
    };

    match rt.block_on(crate::prove(
        &mode,
        &provider,
        transaction_id,
        profile_id,
        cookie,
        access_token,
    )) {
        Ok(_) => ZKP2P_SUCCESS,
        Err(e) => {
            set_last_error(&e.to_string());
            ZKP2P_ERROR_UNKNOWN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn zkp2p_verify(provider: i32, transaction_id: *const c_char) -> i32 {
    let rt = match RUNTIME.get() {
        Some(rt) => rt,
        None => {
            set_last_error("Library not initialized. Call zkp2p_init() first.");
            return ZKP2P_ERROR_INIT;
        }
    };

    let provider = match provider {
        0 => crate::domain::Provider::Wise,
        1 => crate::domain::Provider::PayPal,
        _ => {
            set_last_error("Invalid provider value. Use 0=Wise, 1=PayPal");
            return ZKP2P_ERROR_INVALID;
        }
    };

    let transaction_id = match unsafe { c_str_to_rust_str(transaction_id) } {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid transaction_id string");
            return ZKP2P_ERROR_INVALID;
        }
    };

    match rt.block_on(crate::verify(&provider, transaction_id)) {
        Ok(_) => ZKP2P_SUCCESS,
        Err(e) => {
            set_last_error(&e.to_string());
            ZKP2P_ERROR_UNKNOWN
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn zkp2p_get_last_error() -> *const c_char {
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
pub extern "C" fn zkp2p_free_error_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
