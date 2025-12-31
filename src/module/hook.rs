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
    /// Creates a new `HookContext`.
    ///
    /// # Safety
    /// `def` and `data` must be valid pointers provided by the zsh hook system.
    #[allow(dead_code)]
    pub unsafe fn new(def: *mut bindings::hookdef, data: *mut c_void) -> Self {
        Self {
            raw_def: def,
            raw_data: data,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the name of the hook.
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

    /// # Safety
    /// The caller must ensure that `T` is the correct type and that this access
    /// follows Rust's aliasing rules (only one mutable reference at a time).
    #[allow(clippy::mut_from_ref)] // zshのC APIブリッジであるため抑制
    pub unsafe fn data<T>(&self) -> Option<&mut T> {
        if self.raw_data.is_null() {
            None
        } else {
            // Safety: raw_data is checked for null,
            // type safety is guaranteed by the caller.
            Some(unsafe { &mut *(self.raw_data as *mut T) })
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
            let mut $context = unsafe { $crate::HookContext::new(def, data) };
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
            // 1. まず C API 側の hookdef を取得・チェック
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);

            // ... (既存の重複チェックロジック) ...
            if !hdef.is_null() {
                let funcs_ptr = (*hdef).funcs;
                if !funcs_ptr.is_null() {
                    let mut node = (*funcs_ptr).list.first;
                    while !node.is_null() {
                        if (*node).dat == func as *mut c_void {
                            return Err(HookError::AlreadyExists(name.to_string()));
                        }
                        node = (*node).next;
                    }
                }
            }

            // 2. 対応するシェル配列変数 (例: precmd -> precmd_functions) の同期
            // これにより、zsh本体が「フックがある」と認識してC側のフックも呼ぶようになります
            Self::ensure_zsh_array_initialized(name);

            // 3. C API で関数を登録
            bindings::addhookfunc(c_name.as_ptr() as *mut c_char, Some(func));
        }
        Ok(())
    }

    /// zshのシェル変数(配列)をチェックし、空なら初期化してCフックの発火を促す。
    ///
    /// # Safety
    /// この関数は以下の理由により unsafe です：
    /// 1. zsh 内部のグローバル変数 `paramtab` に直接アクセスする。
    /// 2. `ztrdup`, `setaparam` などの zsh C API を呼び出し、メモリ状態を直接操作する。
    /// 3. 生ポインタのデリファレンスを行う。
    ///    呼び出し側は、この関数が zsh のメインスレッド（または適切なコンテキスト）から
    ///    呼び出されていることを保証する必要があります。
    unsafe fn ensure_zsh_array_initialized(hook_name: &str) {
        let array_name = format!("{}_functions", hook_name);
        let c_array_name = CString::new(array_name).unwrap();

        unsafe {
            // zsh内部から変数を取得
            // paramtab は mutable static なので unsafe ブロック内でのアクセスが必要
            let param =
                bindings::gethashnode2(bindings::paramtab, c_array_name.as_ptr() as *mut c_char)
                    as *mut bindings::param;

            if param.is_null() {
                // 変数自体が存在しない場合は、配列として作成
                let empty_array: [*mut c_char; 2] = [
                    bindings::ztrdup(c_array_name.as_ptr() as *mut c_char),
                    ptr::null_mut(),
                ];
                bindings::setaparam(
                    bindings::ztrdup(c_array_name.as_ptr() as *mut c_char),
                    empty_array.as_ptr() as *mut *mut c_char,
                );
            } else {
                // 変数が存在する場合、中身が空かどうかチェック
                let value = bindings::getaparam(c_array_name.as_ptr() as *mut c_char);
                // rawポインタのデリファレンス (*value) は unsafe
                if value.is_null() || (*value).is_null() {
                    let dummy: [*mut c_char; 2] = [
                        bindings::ztrdup(c_array_name.as_ptr() as *mut c_char),
                        ptr::null_mut(),
                    ];
                    bindings::setaparam(
                        bindings::ztrdup(c_array_name.as_ptr() as *mut c_char),
                        dummy.as_ptr() as *mut *mut c_char,
                    );
                }
            }
        }
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
