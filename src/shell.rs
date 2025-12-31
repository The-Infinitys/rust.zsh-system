//! このモジュールは、Zshシェル内で直接スクリプトを実行するための機能を提供します。
//!
//! RustからZshのコマンドやスクリプトを評価する際に使用されます。
use crate::bindings;
use std::ffi::{CString, c_char};

/// Zshシェル内で指定されたスクリプト文字列を評価（実行）します。
///
/// この関数は、Zshの内部関数 `execstring` を呼び出し、
/// 与えられたスクリプトをあたかもZshのコマンドラインで入力されたかのように実行します。
///
/// # Arguments
/// * `script` - 実行するZshスクリプトを含む文字列。
///
/// # Safety
/// この関数はZshのC APIを呼び出すため `unsafe` な操作を含みます。
/// `script` 文字列は有効なC文字列に変換可能である必要があり、
/// Zshのメモリ管理や実行環境に直接影響を与える可能性があります。
/// 不正なスクリプトを実行すると、Zshセッションのクラッシュや予期しない動作を引き起こす可能性があります。
/// `execstring` の識別名にはクレート名が使用されます。
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
