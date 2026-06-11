use std::ffi::{CStr, CString};
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::types::{FfiConfig, Order, Signal};
use super::Mt4Bridge;

static BRIDGE: OnceLock<Mutex<Option<Mt4Bridge>>> = OnceLock::new();

fn get_bridge() -> Result<MutexGuard<'static, Option<Mt4Bridge>>, &'static str> {
    let lock = BRIDGE.get_or_init(|| Mutex::new(None));
    lock.lock().map_err(|_| "Bridge mutex poisoned")
}

fn to_c_string(s: String) -> *mut std::os::raw::c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn umbra_init(config_json: *const std::os::raw::c_char) -> i32 {
    if config_json.is_null() {
        return -1;
    }
    let c_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let config: FfiConfig = match serde_json::from_str(c_str) {
        Ok(c) => c,
        Err(_) => return -2,
    };
    let mut guard = match get_bridge() {
        Ok(g) => g,
        Err(_) => return -4,
    };
    *guard = Some(Mt4Bridge::with_config(config));
    tracing::info!("[Bridge] Umbra bridge initialized");
    0
}

#[no_mangle]
pub extern "C" fn umbra_analyze(signal_json: *const std::os::raw::c_char) -> *mut std::os::raw::c_char {
    if signal_json.is_null() {
        return to_c_string("{\"error\":\"null signal\"}".into());
    }
    let c_str = match unsafe { CStr::from_ptr(signal_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return to_c_string("{\"error\":\"invalid utf8\"}".into()),
    };
    let signal: Signal = match serde_json::from_str(c_str) {
        Ok(s) => s,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };
    let guard = match get_bridge() {
        Ok(g) => g,
        Err(e) => return to_c_string(format!("{{\"error\":\"{}\"}}", e)),
    };
    let bridge = match guard.as_ref() {
        Some(b) => b,
        None => return to_c_string("{\"error\":\"bridge not initialized\"}".into()),
    };
    let result = bridge.process_signal(signal);
    let output = match &result {
        Ok(order) => serde_json::to_string(order).unwrap_or_else(|_| "{\"error\":\"serialization failed\"}".into()),
        Err(e) => format!("{{\"error\":\"{}\"}}", e),
    };
    to_c_string(output)
}

#[no_mangle]
pub extern "C" fn umbra_execute(order_json: *const std::os::raw::c_char) -> i32 {
    if order_json.is_null() {
        return -1;
    }
    let c_str = match unsafe { CStr::from_ptr(order_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let order: Order = match serde_json::from_str(c_str) {
        Ok(o) => o,
        Err(_) => return -2,
    };
    let guard = match get_bridge() {
        Ok(g) => g,
        Err(_) => return -4,
    };
    let bridge = match guard.as_ref() {
        Some(b) => b,
        None => return -3,
    };
    tracing::info!("[Bridge] Order executed: {} {} {}", order.symbol, order.action, order.volume);
    bridge.executor.confirm_order(order);
    0
}

#[no_mangle]
pub extern "C" fn umbra_shutdown() {
    if let Ok(mut guard) = get_bridge() {
        *guard = None;
    }
    tracing::info!("[Bridge] Umbra bridge shutdown");
}

#[no_mangle]
pub extern "C" fn umbra_free_string(s: *mut std::os::raw::c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

pub struct FfiBridge;

impl FfiBridge {
    pub fn new() -> Self {
        Self
    }
}

/// Call MT5 terminal via the bridge. Sends a JSON request and returns JSON response.
pub fn call_mt5(method: &str, params: &str) -> anyhow::Result<String> {
    let request = format!("{{\"method\":\"{}\",\"params\":{}}}", method, params);
    let c_request = std::ffi::CString::new(request)?;
    let result = unsafe { umbra_mt5_call(c_request.as_ptr()) };
    if result.is_null() {
        anyhow::bail!("MT5 call returned null");
    }
    let c_str = unsafe { std::ffi::CStr::from_ptr(result) };
    let response = c_str.to_str()?.to_string();
    unsafe { umbra_free_string(result) };
    Ok(response)
}

extern "C" {
    fn umbra_mt5_call(request: *const std::os::raw::c_char) -> *mut std::os::raw::c_char;
}
