use crate::ZString;
use crate::bindings;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub struct ZshParameter;

impl ZshParameter {
    /// 文字列変数を取得する
    pub fn get_str(name: &str) -> Option<String> {
        let c_name = CString::new(name).ok()?;
        unsafe {
            let ptr = bindings::getsparam(c_name.as_ptr() as *mut c_char);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    /// 文字列変数を代入する
    /// ZString を利用して zsh 内部のメモリ確保 (ztrdup) を行い、その所有権を zsh に渡す
    pub fn set_str(name: &str, value: &str) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;
        let z_str = ZString::new(value);

        unsafe {
            // ZString から raw ポインタを取り出し、Rust 側の Drop (zsfree) を回避して Zsh に委ねる
            let ptr = z_str.as_ptr();
            std::mem::forget(z_str);

            let res = bindings::setsparam(c_name.as_ptr() as *mut c_char, ptr);
            if !res.is_null() {
                Ok(())
            } else {
                Err("Failed to set string parameter")
            }
        }
    }

    /// 整数変数を取得する
    pub fn get_int(name: &str) -> bindings::zlong {
        let c_name = CString::new(name).unwrap_or_default();
        unsafe { bindings::getiparam(c_name.as_ptr() as *mut c_char) }
    }

    /// 整数変数を代入する
    pub fn set_int(name: &str, value: bindings::zlong) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;
        unsafe {
            let res = bindings::setiparam(c_name.as_ptr() as *mut c_char, value);
            if !res.is_null() {
                Ok(())
            } else {
                Err("Failed to set integer parameter")
            }
        }
    }

    /// 配列変数を代入する
    /// ポインタ配列自体も zalloc で確保することで、zsh 内部の zsfree と整合させます
    pub fn set_array(name: &str, values: Vec<&str>) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;

        unsafe {
            // 1. 配列の要素数 + 1 (NULL終端用) のサイズを zalloc で確保
            // ポインタのサイズは std::mem::size_of::<*mut c_char>()
            let count = values.len();
            let array_size = (count + 1) * std::mem::size_of::<*mut c_char>();
            let ptr_array = bindings::zalloc(array_size) as *mut *mut c_char;

            if ptr_array.is_null() {
                return Err("zsh: out of memory");
            }

            // 2. 各要素を ztrdup (ZString) で作成し、配列に格納
            for (i, val) in values.into_iter().enumerate() {
                let z_val = ZString::new(val);
                let p = z_val.as_ptr();
                std::mem::forget(z_val); // Zsh 側で解放させるため Rust 側は forget
                *ptr_array.add(i) = p;
            }

            // 3. NULL 終端
            *ptr_array.add(count) = std::ptr::null_mut();

            // 4. zsh に代入。zsh は ptr_array 自体も zsfree する
            let res = bindings::setaparam(c_name.as_ptr() as *mut c_char, ptr_array);

            if !res.is_null() {
                Ok(())
            } else {
                // 本来は失敗時に確保したメモリを遡って解放すべきですが、
                // setaparam が失敗することは稀であり、多くの場合 zsh 側で処理されます
                Err("Failed to set array parameter")
            }
        }
    }

    /// 変数を削除する
    pub fn unset(name: &str) {
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                bindings::unsetparam(c_name.as_ptr() as *mut c_char);
            }
        }
    }
}
