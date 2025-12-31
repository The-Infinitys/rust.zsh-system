//! このモジュールは、Zshのフックシステムと安全にインタラクトするための機能を提供します。
//!
//! Zshのフックは特定のイベント（例: コマンド実行前、プロンプト表示前）でカスタム関数を実行することを可能にし、
//! このモジュールはRust関数をZshフックとして登録・実行・管理するための安全なインターフェースを提供します。
use crate::bindings;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::{self, addr_of_mut};
use thiserror::Error;

/// フック実行時のコンテキストを安全にラップする構造体。
///
/// Zshから提供されるフック定義 (`hookdef`) とカスタムデータ (`data`) への参照を保持します。
#[allow(dead_code)] // 構造体全体への警告を抑制
pub struct HookContext<'a> {
    #[allow(dead_code)]
    raw_def: *mut bindings::hookdef,
    #[allow(dead_code)]
    raw_data: *mut c_void,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> HookContext<'a> {
    /// 新しい `HookContext` インスタンスを作成します。
    ///
    /// # Safety
    /// `def` と `data` は、Zshのフックシステムによって提供された有効なポインタである必要があります。
    /// この関数は生のポインタを扱うため `unsafe` です。
    #[allow(dead_code)]
    pub unsafe fn new(def: *mut bindings::hookdef, data: *mut c_void) -> Self {
        Self {
            raw_def: def,
            raw_data: data,
            _marker: std::marker::PhantomData,
        }
    }

    /// フックの名前を返します。
    ///
    /// # Safety
    /// 内部的に生のポインタ (`raw_def`, `name`) をデリファレンスするため、
    /// これらのポインタが有効であることを呼び出し元が保証する必要があります。
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

    /// フックに関連付けられたカスタムデータへの可変参照を取得します。
    ///
    /// # Safety
    /// 呼び出し元は `T` が正しい型であることを保証し、
    /// Rustのエイリアシングルール（一度に一つの可変参照のみ）に従う必要があります。
    /// データポインタ `raw_data` が有効な `T` 型のインスタンスを指していることを保証しなければなりません。
    #[allow(clippy::mut_from_ref)] // ZshのC APIブリッジであるため抑制
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

/// Zshフックハンドラ関数を定義するためのマクロ。
///
/// このマクロは、ZshのC APIからのコールバック関数として機能する`unsafe extern "C" fn`を生成します。
/// 生成された関数は、`HookContext`を通じてフックの定義と関連データにアクセスし、
/// ユーザーが定義したRustのクロージャ`$body`を実行します。
///
/// # 引数
/// - `$name`: 生成されるC関数の名前。
/// - `$context`: `$body`内で使用できる`HookContext`の変数名。
/// - `$body`: フックが実行されたときに実行されるRustコードブロック。
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

/// フック操作中に発生する可能性のあるエラーを定義する列挙型。
#[derive(Debug, Error)]
pub enum HookError {
    /// 指定されたフックに同じ関数が既に登録されている場合に発生します。
    #[error("Hook '{0}' already has this function registered")]
    AlreadyExists(String),
    /// 指定されたフックが見つからない場合に発生します。
    #[error("Hook '{0}' does not exist")]
    NotFound(String),
    /// 文字列変換に失敗した場合に発生します（例: nullバイトを含む文字列）。
    #[error("Failed to process string conversion")]
    InvalidString,
}

/// Zshフックハンドラとして登録されるC互換関数の型エイリアス。
///
/// Zshの`hookdef`ポインタとカスタムデータポインタを受け取り、`i32`を返します。
pub type ZshHookFn = unsafe extern "C" fn(arg1: *mut bindings::hookdef, arg2: *mut c_void) -> i32;

/// Zshフックシステムとインタラクトするための静的ユーティリティ。
///
/// フックのリスト取得、追加、削除、実行などの機能を提供します。
pub struct Hook;

impl Hook {
    /// Zshに現在登録されている全てのフックの名前をリストアップします。
    ///
    /// # Safety
    /// Zsh内部のグローバル変数`zshhooks`への生ポインタアクセスを含むため、
    /// Zshのメモリレイアウトと構造体を信頼しています。
    pub fn list() -> Vec<String> {
        let mut names = Vec::new();
        unsafe {
            // [hookdef; 0] の先頭ポインタを取得 (Zshのフック定義配列の開始)
            let mut ptr = addr_of_mut!(bindings::zshhooks) as *mut bindings::hookdef;

            if ptr.is_null() {
                return names;
            }

            // Zshの構造に従い、nameフィールドがNULLになるまでループ
            // ※ センチネル（終端要素）に到達するまでポインタをずらす
            while !ptr.is_null() && !(*ptr).name.is_null() {
                let name_ptr = (*ptr).name;
                if let Ok(name) = CStr::from_ptr(name_ptr).to_str() {
                    names.push(name.to_string());
                }
                // 次の要素（hookdef構造体のサイズ分）進める
                ptr = ptr.add(1);
            }
        }
        names
    }

    /// 指定された名前のフックにRust関数を登録します。
    ///
    /// # Arguments
    /// - `name`: 登録するフックの名前。
    /// - `func`: フックイベント発生時に呼び出されるC互換のRust関数。
    ///
    /// # Errors
    /// - `HookError::InvalidString`: `name`の文字列変換に失敗した場合。
    /// - `HookError::AlreadyExists`: 同じ名前のフックに同じ関数が既に登録されている場合。
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
            // Zshの `addhookfunc` を呼び出して関数を登録
            bindings::addhookfunc(c_name.as_ptr() as *mut c_char, Some(func));
        }
        Ok(())
    }

    /// 指定された名前のフックからRust関数を削除します。
    ///
    /// # Arguments
    /// - `name`: フックの名前。
    /// - `func`: 削除するC互換のRust関数。
    ///
    /// # Errors
    /// - `HookError::InvalidString`: `name`の文字列変換に失敗した場合。
    /// - `HookError::NotFound`: 指定されたフックまたは関数が見つからない場合。
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

            // Zshの `deletehookfunc` を呼び出して関数を削除
            bindings::deletehookfunc(c_name.as_ptr() as *mut c_char, Some(func));
        }
        Ok(())
    }

    /// 指定された名前のフックを引数なしで実行します。
    ///
    /// # Arguments
    /// - `name`: 実行するフックの名前。
    ///
    /// # Errors
    /// - `HookError::InvalidString`: `name`の文字列変換に失敗した場合。
    /// - `HookError::NotFound`: 指定されたフックが見つからない場合。
    pub fn run(name: &str) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;
        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);
            if hdef.is_null() {
                return Err(HookError::NotFound(name.to_string()));
            }
            // Zshの `runhookdef` を呼び出し、NULLデータポインタで実行
            bindings::runhookdef(hdef, ptr::null_mut());
        }
        Ok(())
    }

    /// 指定された名前のフックをカスタムデータと共に実行します。
    ///
    /// # Arguments
    /// - `name`: 実行するフックの名前。
    /// - `data`: フックハンドラに渡されるカスタムデータへの可変参照。
    ///
    /// # Errors
    /// - `HookError::InvalidString`: `name`の文字列変換に失敗した場合。
    /// - `HookError::NotFound`: 指定されたフックが見つからない場合。
    ///
    /// # Safety
    /// `data`引数はフックハンドラ内で `*mut T` として扱われるため、
    /// 呼び出し元はフックハンドラが正しい型 `T` でデータを安全にデリファレンスできることを保証する必要があります。
    pub fn run_with_data<T>(name: &str, data: &mut T) -> Result<(), HookError> {
        let c_name = CString::new(name).map_err(|_| HookError::InvalidString)?;
        unsafe {
            let hdef = bindings::gethookdef(c_name.as_ptr() as *mut c_char);
            if hdef.is_null() {
                return Err(HookError::NotFound(name.to_string()));
            }
            // Zshの `runhookdef` を呼び出し、カスタムデータポインタで実行
            bindings::runhookdef(hdef, data as *mut T as *mut c_void);
        }
        Ok(())
    }
}
