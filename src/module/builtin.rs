//! このモジュールは、ZshのビルトインコマンドをRustで定義および管理するための機能を提供します。
//!
//! Zshのビルトインコマンドは、`BuiltinHandler`トレイトによって定義されたRust関数として実装され、
//! `Builtin`構造体を通じてZshに登録されます。
//! 登録されたハンドラは、`dispatch`関数を通じて実行時に呼び出されます。
use crate::bindings;
use crate::zalloc::ZString;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Rust側で定義する安全なビルトインコマンドハンドラの型エイリアス。
///
/// コマンド名と引数のスライスを受け取り、Zshの終了ステータスとして`i32`を返します。
pub type BuiltinHandler = fn(name: &str, args: &[&str]) -> i32;

/// Zshのビルトインコマンドの定義をカプセル化する構造体。
///
/// コマンド名、ハンドラ関数、最小・最大引数を保持します。
pub struct Builtin {
    name: ZString,
    handler: BuiltinHandler,
    min_args: i32,
    max_args: i32,
}

impl Builtin {
    /// 新しい `Builtin` インスタンスを作成します。
    ///
    /// `name` はビルトインコマンドの名前、`handler` は実行時に呼び出されるRust関数です。
    pub fn new(name: &str, handler: BuiltinHandler) -> Self {
        Self {
            name: ZString::new(name),
            handler,
            min_args: 0,
            max_args: -1, // -1 means no maximum argument limit
        }
    }

    /// Zshから直接呼ばれるC互換のブリッジ関数。
    ///
    /// Zshのビルトインコマンドのハンドラシグネチャに合わせ、
    /// Rustの安全な`BuiltinHandler`を呼び出す役割を担います。
    /// 引数のポインタをRustの安全な型に変換し、登録されたハンドラにディスパッチします。
    ///
    /// # Safety
    /// ZshのC APIからの生ポインタ (`name`, `argv`) を扱うため、
    /// ポインタが有効であり、ZshのAPI規約に従って使用されることを呼び出し元が保証する必要があります。
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

        // 3. 登録されたハンドラを呼び出す
        dispatch(name_str, &args)
    }

    /// `Builtin`インスタンスをZshの`builtin`構造体として表現します。
    ///
    /// この生構造体はZshのモジュールAPIに渡され、コマンドとして登録されます。
    ///
    /// # Safety
    /// `std::mem::zeroed()` を使用して構造体をゼロ初期化し、
    /// 生ポインタの操作が含まれるため、この関数は`unsafe`です。
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

    /// `Builtin`に格納されているRustハンドラ関数を取得します。
    ///
    /// これは主に内部的な使用とコンパイラの警告回避のためです。
    pub fn handler(&self) -> BuiltinHandler {
        self.handler
    }
}

// 静的変数による管理
use std::sync::Mutex;
/// 登録されたビルトインハンドラをグローバルに管理するためのミューテックス保護されたベクタ。
/// (`'static str`, `BuiltinHandler`) のタプルを格納します。
static HANDLERS: Mutex<Vec<(&'static str, BuiltinHandler)>> = Mutex::new(Vec::new());

/// ビルトインコマンドのハンドラをグローバルディスパッチャに登録します。
///
/// 同じ名前のハンドラが既に登録されている場合は、重複して登録されません。
pub fn register_handler(name: &'static str, handler: BuiltinHandler) {
    if let Ok(mut h) = HANDLERS.lock() {
        // 同じ名前が既にあるかチェックして重複を防ぐ
        if !h.iter().any(|(n, _)| *n == name) {
            h.push((name, handler));
        }
    }
}

/// 指定された名前のビルトインハンドラを実行します。
///
/// Zshの`bridge_handler`から呼び出され、適切なRustハンドラ関数に処理を委譲します。
/// ハンドラが見つからない場合は終了ステータス `1` を返します。
pub fn dispatch(name: &str, args: &[&str]) -> i32 {
    if let Ok(h_list) = HANDLERS.lock()
        && let Some((_, h)) = h_list.iter().find(|(n, _)| *n == name)
    {
        return h(name, args);
    }
    1
}
