use crate::bindings;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use thiserror::Error;

/// フック実行時のコンテキストを保持する安全なラッパー
/// フック実行時のコンテキストを保持する安全なラッパー
#[allow(dead_code)] // 構造体全体への警告を抑制
pub struct HookContext<'a> {
    #[allow(dead_code)]
    raw_def: *mut bindings::hookdef,
    #[allow(dead_code)]
    raw_data: *mut c_void,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> HookContext<'a> {
    /// マクロから呼び出されるため、不使用警告を抑制
    #[allow(dead_code)]
    pub unsafe fn new(def: *mut bindings::hookdef, data: *mut c_void) -> Self {
        Self {
            raw_def: def,
            raw_data: data,
            _marker: std::marker::PhantomData,
        }
    }

    /// フックの名前を取得する
    #[allow(dead_code)] // マクロ内での使用を考慮
    pub fn hook_name(&self) -> &str {
        if self.raw_def.is_null() {
            return "";
        }
        unsafe {
            let name_ptr = (*self.raw_def).name;
            if name_ptr.is_null() {
                ""
            } else {
                CStr::from_ptr(name_ptr).to_str().unwrap_or("")
            }
        }
    }

    /// フックに渡されたデータを特定の型として取得する
    #[allow(dead_code)]
    pub unsafe fn data<T>(&self) -> Option<&mut T> {
        if self.raw_data.is_null() {
            None
        } else {
            unsafe { Some(&mut *(self.raw_data as *mut T)) }
        }
    }
}

/// zsh ハンドラを安全に定義するためのマクロ
#[macro_export]
macro_rules! zsh_hook_handler {
    ($name:ident, $context:ident, $body:block) => {
        pub unsafe extern "C" fn $name(
            def: *mut $crate::bindings::hookdef,
            data: *mut ::std::os::raw::c_void,
        ) -> i32 {
            // コンテキストの生成（unsafe ブロックで囲む）
            let mut $context = unsafe { $crate::module::hook::HookContext::new(def, data) };
            let mut handler = || -> i32 { $body };
            handler()
        }
    };
}

#[derive(Debug, Error)]
pub enum HookError {
    #[error("Hook '{0}' already has this function registered")]
    AlreadyExists(String),
    #[error("Hook '{0}' does not exist")]
    NotFound(String),
    #[error("Failed to process string conversion")]
    InvalidString,
}

pub type ZshHookFn = unsafe extern "C" fn(arg1: *mut bindings::hookdef, arg2: *mut c_void) -> i32;

pub struct Hook;

impl Hook {
    pub fn add(name: &str, func: ZshHookFn) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;

        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);

            if !hdef.is_null() {
                // 重複した unsafe ブロックを削除し、一重のブロックで管理
                let funcs_ptr = (*hdef).funcs;
                if !funcs_ptr.is_null() {
                    let list_root = (*funcs_ptr).list;
                    let mut node = list_root.first;

                    while !node.is_null() {
                        let registered_func = (*node).dat;
                        if registered_func == func as *mut c_void {
                            return Err(HookError::AlreadyExists(name.to_string()));
                        }
                        node = (*node).next;
                    }
                }
            }
            bindings::addhookfunc(c_name.as_ptr() as *mut c_char, Some(func));
        }
        Ok(())
    }

    pub fn remove(name: &str, func: ZshHookFn) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;

        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);
            if hdef.is_null() {
                return Err(HookError::NotFound(name.to_string()));
            }

            let funcs_ptr = (*hdef).funcs;
            let mut found = false;
            if !funcs_ptr.is_null() {
                let mut node = (*funcs_ptr).list.first;
                while !node.is_null() {
                    if (*node).dat == func as *mut c_void {
                        found = true;
                        break;
                    }
                    node = (*node).next;
                }
            }

            if !found {
                return Err(HookError::NotFound(format!("Function in hook '{}'", name)));
            }

            bindings::deletehookfunc(c_name.as_ptr() as *mut c_char, Some(func));
        }
        Ok(())
    }

    pub fn run(name: &str) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;
        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);
            if hdef.is_null() {
                return Err(HookError::NotFound(name.to_string()));
            }
            bindings::runhookdef(hdef, ptr::null_mut());
        }
        Ok(())
    }

    pub fn run_with_data<T>(name: &str, data: &mut T) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;
        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);
            if hdef.is_null() {
                return Err(HookError::NotFound(name.to_string()));
            }
            bindings::runhookdef(hdef, data as *mut T as *mut c_void);
        }
        Ok(())
    }
}
