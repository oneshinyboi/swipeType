//! C FFI interface for the SwipeEngine

use once_cell::sync::Lazy;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::SwipeEngine;

static ENGINE: Lazy<Mutex<SwipeEngine>> = Lazy::new(|| Mutex::new(SwipeEngine::new()));

/// Returns the number of words loaded, or -1 on error.
#[no_mangle]
pub extern "C" fn swipe_engine_load_dictionary(path: *const c_char) -> i32 {
    if path.is_null() {
        return -1;
    }

    let path_str = unsafe {
        match CStr::from_ptr(path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let content = match std::fs::read_to_string(path_str) {
        Ok(c) => c,
        Err(_) => return -1,
    };

    let mut engine = match ENGINE.lock() {
        Ok(e) => e,
        Err(_) => return -1,
    };

    engine.load_dictionary_from_text(&content);
    engine.word_count() as i32
}

/// Returns the number of words loaded, or -1 on error.
#[no_mangle]
pub extern "C" fn swipe_engine_load_dictionary_str(content: *const c_char) -> i32 {
    if content.is_null() {
        return -1;
    }

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let mut engine = match ENGINE.lock() {
        Ok(e) => e,
        Err(_) => return -1,
    };

    engine.load_dictionary_from_text(content_str);
    engine.word_count() as i32
}

#[no_mangle]
pub extern "C" fn swipe_engine_word_count() -> i32 {
    match ENGINE.lock() {
        Ok(e) => e.word_count() as i32,
        Err(_) => -1,
    }
}

/// Returns a JSON string with predictions array. Caller must free with swipe_engine_free_string.
#[no_mangle]
pub extern "C" fn swipe_engine_predict(input: *const c_char, limit: i32) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let input_str = unsafe {
        match CStr::from_ptr(input).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let engine = match ENGINE.lock() {
        Ok(e) => e,
        Err(_) => return std::ptr::null_mut(),
    };

    let predictions = engine.predict(input_str, limit.max(0) as usize);

    let mut json = String::from("[");
    for (i, pred) in predictions.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        json.push_str(r#"{"word":""#);
        for ch in pred.word.chars() {
            match ch {
                '"' => json.push_str("\\\""),
                '\\' => json.push_str("\\\\"),
                '\n' => json.push_str("\\n"),
                '\r' => json.push_str("\\r"),
                '\t' => json.push_str("\\t"),
                _ => json.push(ch),
            }
        }
        json.push_str(&format!(
            r#"","score":{:.4},"freq":{:.4}}}"#,
            pred.score, pred.freq
        ));
    }
    json.push(']');

    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn swipe_engine_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

#[no_mangle]
pub extern "C" fn swipe_engine_set_pop_weight(weight: f64) {
    if let Ok(mut engine) = ENGINE.lock() {
        engine.set_pop_weight(weight);
    }
}
