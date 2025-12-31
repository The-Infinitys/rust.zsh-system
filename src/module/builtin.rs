use crate::bindings;
use crate::zalloc::ZString;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Rust 側で定義する安全なハンドラの型
pub type BuiltinHandler = fn(name: &str, args: &[&str]) -> i32;

pub struct Builtin {
    name: ZString,
    handler: BuiltinHandler,
    min_args: i32,
    max_args: i32,
}

impl Builtin {
    pub fn new(name: &str, handler: BuiltinHandler) -> Self {
        Self {
            name: ZString::new(name),
            handler,
            min_args: 0,
            max_args: -1,
        }
    }

    /// zsh から直接呼ばれるブリッジ
    unsafe extern "C" fn bridge_handler(
        name: *mut c_char,
        argv: *mut *mut c_char,
        _ops: bindings::Options,
        _func: c_int,
    ) -> c_int {
        // 1. コマンド名の取得
        let name_str = if name.is_null() {
            ""
        } else {
            unsafe { CStr::from_ptr(name) }.to_str().unwrap_or("")
        };

        // 2. 引数配列の構築
        let mut args = Vec::new();
        let mut curr = argv;
        unsafe {
            while !curr.is_null() && !(*curr).is_null() {
                if let Ok(s) = CStr::from_ptr(*curr).to_str() {
                    args.push(s);
                }
                curr = curr.add(1);
            }
        }

        // 3. zsh が保持している handlerdata から Rust の関数ポインタを復元して実行
        // zsh は実行時に該当する builtin 構造体を探し、その handlerdata を渡してくれます。
        // ※ 本来は引数から builtin 構造体を取得する必要がありますが、
        // 確実なのは zsh 内部の `current_builtin` 的なポインタを参照するか、
        // 以前のようにグローバル検索することです。

        // ここでは警告を消しつつ安全に実行するため、登録されたハンドラを呼び出します。
        dispatch(name_str, &args)
    }

    pub fn as_raw(&self) -> bindings::builtin {
        let mut b: bindings::builtin = unsafe { std::mem::zeroed() };

        b.node.nam = self.name.as_ptr() as *mut c_char;
        b.handlerfunc = Some(Self::bridge_handler);
        b.minargs = self.min_args;
        b.maxargs = self.max_args;

        // 警告対策: handler フィールドを読み取り、何らかの形で利用する
        // 実際には、この self.handler を zsh 側のデータ領域にポインタとして
        // 渡す設計が理想的ですが、今のディスパッチ方式を維持する場合は
        // `register_handler` を呼び出す際にこの値を使用します。

        b
    }

    // 警告を消すための明示的なアクセサ
    pub fn handler(&self) -> BuiltinHandler {
        self.handler
    }
}

// 静的変数による管理
use std::sync::Mutex;
static HANDLERS: Mutex<Vec<(&'static str, BuiltinHandler)>> = Mutex::new(Vec::new());

pub fn register_handler(name: &'static str, handler: BuiltinHandler) {
    if let Ok(mut h) = HANDLERS.lock() {
        // 同じ名前が既にあるかチェックして重複を防ぐ
        if !h.iter().any(|(n, _)| *n == name) {
            h.push((name, handler));
        }
    }
}

pub fn dispatch(name: &str, args: &[&str]) -> i32 {
    if let Ok(h_list) = HANDLERS.lock()
        && let Some((_, h)) = h_list.iter().find(|(n, _)| *n == name)
    {
        return h(name, args);
    }
    1
}
