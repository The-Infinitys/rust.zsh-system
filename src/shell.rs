use crate::bindings;
use std::ffi::{CString, c_char};

pub fn eval(script: &str) {
    let c_str = CString::new(script).unwrap_or_else(|_| CString::new("").unwrap());

    // クレート名をデバッグ識別名として取得 (例: "zsh-infinite")
    let crate_name = env!("CARGO_PKG_NAME");
    let c_name = CString::new(crate_name).unwrap_or_else(|_| CString::new("zsh-module").unwrap());

    unsafe {
        // zsh内部の execstring 関数を呼び出す
        bindings::execstring(
            c_str.as_ptr() as *mut c_char,
            0,                              // flags
            0,                              // dont_hist
            c_name.as_ptr() as *mut c_char, // 識別名
        );
    }
}
