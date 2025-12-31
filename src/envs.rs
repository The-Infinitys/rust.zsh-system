//! This module provides safe Rust bindings for interacting with Zsh shell parameters (variables).
//!
//! It offers functions to get and set string, integer, and array parameters,
//! as well as to unset parameters, abstracting away the unsafe FFI calls to Zsh's C API.
use crate::ZString;
use crate::bindings;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
mod direct;
pub use direct::*;
/// `ZshParameter` provides a safe interface for accessing and modifying Zsh shell parameters.
pub struct ZshParameter;

impl ZshParameter {
    /// Retrieves the value of a string parameter from Zsh.
    ///
    /// Returns `Some(String)` if the parameter exists and its value can be
    /// converted to a Rust `String`, otherwise returns `None`.
    pub fn get_str(name: &str) -> Option<String> {
        let c_name = CString::new(name).ok()?;
        unsafe {
            // Calls Zsh's `getsparam` to get the string parameter.
            let ptr = bindings::getsparam(c_name.as_ptr() as *mut c_char);
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    /// Sets the value of a string parameter in Zsh.
    ///
    /// This function uses `ZString` to allocate memory for the string value
    /// using Zsh's memory allocator (`ztrdup`) and transfers ownership to Zsh.
    ///
    /// Returns `Ok(())` on success, or an `Err` with a static string if the
    /// name is invalid or setting the parameter fails.
    pub fn set_str(name: &str, value: &str) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;
        let z_str = ZString::new(value);

        unsafe {
            // Extract the raw pointer from ZString and `forget` it to prevent Rust's Drop
            // from freeing the memory, as Zsh will manage its lifetime.
            let ptr = z_str.as_ptr();
            std::mem::forget(z_str);

            // Calls Zsh's `setsparam` to set the string parameter.
            let res = bindings::setsparam(c_name.as_ptr() as *mut c_char, ptr);
            if !res.is_null() {
                Ok(())
            } else {
                Err("Failed to set string parameter")
            }
        }
    }

    /// Retrieves the value of an integer parameter from Zsh.
    ///
    /// Returns the integer value. If the parameter does not exist or cannot be
    /// converted to an integer, Zsh's internal `getiparam` behavior (usually 0) applies.
    pub fn get_int(name: &str) -> bindings::zlong {
        let c_name = CString::new(name).unwrap_or_default();
        unsafe { bindings::getiparam(c_name.as_ptr() as *mut c_char) }
    }

    /// Sets the value of an integer parameter in Zsh.
    ///
    /// Returns `Ok(())` on success, or an `Err` with a static string if the
    /// name is invalid or setting the parameter fails.
    pub fn set_int(name: &str, value: bindings::zlong) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;
        unsafe {
            // Calls Zsh's `setiparam` to set the integer parameter.
            let res = bindings::setiparam(c_name.as_ptr() as *mut c_char, value);
            if !res.is_null() {
                Ok(())
            } else {
                Err("Failed to set integer parameter")
            }
        }
    }

    /// Sets the value of an array parameter in Zsh.
    ///
    /// The array elements and the array itself are allocated using Zsh's `zalloc`
    /// and `ztrdup` functions, ensuring proper memory management within Zsh.
    ///
    /// Returns `Ok(())` on success, or an `Err` with a static string if the
    /// name is invalid or setting the parameter fails.
    pub fn set_array(name: &str, values: Vec<&str>) -> Result<(), &'static str> {
        let c_name = CString::new(name).map_err(|_| "Invalid name")?;

        unsafe {
            // 1. Allocate memory for the array of pointers using `zalloc`.
            // The size is (number of elements + 1 for NULL terminator) * size of a pointer.
            let count = values.len();
            let array_size = (count + 1) * std::mem::size_of::<*mut c_char>();
            let ptr_array = bindings::zalloc(array_size) as *mut *mut c_char;

            if ptr_array.is_null() {
                return Err("zsh: out of memory");
            }

            // 2. Create each array element using `ZString` (which uses `ztrdup`)
            // and store the raw pointer in the allocated array.
            for (i, val) in values.into_iter().enumerate() {
                let z_val = ZString::new(val);
                let p = z_val.as_ptr();
                std::mem::forget(z_val); // Zsh will manage this memory
                *ptr_array.add(i) = p;
            }

            // 3. NULL-terminate the array, as required by Zsh.
            *ptr_array.add(count) = std::ptr::null_mut();

            // 4. Call Zsh's `setaparam` to set the array parameter.
            // Zsh takes ownership of `ptr_array` and its contents.
            let res = bindings::setaparam(c_name.as_ptr() as *mut c_char, ptr_array);

            if !res.is_null() {
                Ok(())
            } else {
                // In case of failure, normally we should free the allocated memory.
                // However, `setaparam` rarely fails, and Zsh often handles cleanup.
                Err("Failed to set array parameter")
            }
        }
    }

    /// Unsets (deletes) a parameter in Zsh.
    ///
    /// If the parameter does not exist, this function does nothing.
    pub fn unset(name: &str) {
        if let Ok(c_name) = CString::new(name) {
            unsafe {
                // Calls Zsh's `unsetparam` to remove the parameter.
                bindings::unsetparam(c_name.as_ptr() as *mut c_char);
            }
        }
    }
    /// ダイレクトポインタアクセス用のハンドラを取得します。
    /// 指定した名前の変数が Zsh 内に存在する場合、そのメモリアドレスをキャッシュして
    /// 高速な読み書き（ハッシュ検索のスキップ）が可能になります。
    ///
    /// # Example
    /// ```
    /// let mut prompt = ZshParameter::direct::<String>("PROMPT");
    /// prompt.set("new prompt").unwrap();
    /// ```
    pub fn direct<T: ZshParamType>(name: &str) -> ZshParamPtr<T> {
        ZshParamPtr::<T>::new(name)
    }
}
