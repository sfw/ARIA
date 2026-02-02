//! String-keyed map runtime support for FORMA

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

/// Internal representation of a FORMA string map.
pub struct FormaMap {
    inner: HashMap<String, String>,
}

/// Create a new empty map.
#[no_mangle]
pub extern "C" fn forma_map_new() -> *mut FormaMap {
    let m = Box::new(FormaMap {
        inner: HashMap::new(),
    });
    Box::into_raw(m)
}

/// Return the number of entries in the map.
#[no_mangle]
pub extern "C" fn forma_map_len(m: *const FormaMap) -> i64 {
    if m.is_null() {
        return 0;
    }
    unsafe { (*m).inner.len() as i64 }
}

/// Get the value for a key. Returns a newly allocated C string (caller must free with forma_str_free),
/// or null if the key is not present.
#[no_mangle]
pub extern "C" fn forma_map_get(m: *const FormaMap, key: *const c_char) -> *mut c_char {
    if m.is_null() || key.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        let key_str = CStr::from_ptr(key).to_string_lossy();
        match (*m).inner.get(key_str.as_ref()) {
            Some(val) => CString::new(val.as_str()).unwrap_or_default().into_raw(),
            None => ptr::null_mut(),
        }
    }
}

/// Set a key-value pair in the map. Both key and value are C strings.
#[no_mangle]
pub extern "C" fn forma_map_set(m: *mut FormaMap, key: *const c_char, value: *const c_char) {
    if m.is_null() || key.is_null() || value.is_null() {
        return;
    }
    unsafe {
        let key_str = CStr::from_ptr(key).to_string_lossy().into_owned();
        let val_str = CStr::from_ptr(value).to_string_lossy().into_owned();
        (*m).inner.insert(key_str, val_str);
    }
}

/// Check whether the map contains a given key.
#[no_mangle]
pub extern "C" fn forma_map_contains(m: *const FormaMap, key: *const c_char) -> bool {
    if m.is_null() || key.is_null() {
        return false;
    }
    unsafe {
        let key_str = CStr::from_ptr(key).to_string_lossy();
        (*m).inner.contains_key(key_str.as_ref())
    }
}

/// Remove a key from the map. Returns true if the key was present.
#[no_mangle]
pub extern "C" fn forma_map_remove(m: *mut FormaMap, key: *const c_char) -> bool {
    if m.is_null() || key.is_null() {
        return false;
    }
    unsafe {
        let key_str = CStr::from_ptr(key).to_string_lossy().into_owned();
        (*m).inner.remove(&key_str).is_some()
    }
}

/// Free the map and all its contents.
#[no_mangle]
pub extern "C" fn forma_map_free(m: *mut FormaMap) {
    if m.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(m));
    }
}
