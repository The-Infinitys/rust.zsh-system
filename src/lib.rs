//! `zsh-system` は Zsh (Z Shell) モジュールを Rust で開発するためのライブラリです。
//!
//! このクレートは、Zsh の C API への安全なラッパーを提供し、
//! ビルトインコマンド、フック、パラメータなど、Zsh の機能を Rust から利用できるようにします。
//!
//! 主な機能:
//! - Zsh シェルパラメータ (`ZshParameter`) の取得・設定。
//! - Zsh のメモリ管理 (`ZBox`, `ZString`) を利用した安全なメモリ操作。
//! - Zsh の `Module` システムへの統合を簡素化するマクロ (`export_module!`)。
//! - ビルトインコマンド、条件定義、数式関数、パラメータ定義などの Zsh 機能の登録。
//! - Zsh のフックシステム (`Hook`) とのインタラクション。
//! - Zsh コマンド (`shell::eval`) の実行。
mod envs;
mod macros;
mod module;
mod shell;
mod zalloc;
pub use crate::module::*;
pub use envs::*;
pub use shell::*;
pub use zalloc::*;
/// Zsh C APIへのFFIバインディングが含まれています。`build.rs`によって生成されます。
#[doc(hidden)]
pub mod bindings;

/// `export_module!`マクロから内部的にZshとのブリッジとして利用されるAPI。
/// 直接利用することは想定されていません。
#[doc(hidden)]
pub mod __private_api {
    use crate::Features;
    use crate::bindings;

    /// Zshがモジュールから提供される「機能の名前リスト」を取得するためのブリッジ。
    ///
    /// # Safety
    /// Zshからの生ポインタ (`m`, `out`) を扱うため、呼び出し元はZshのAPI規約に従う必要があります。
    /// 特に `out` には有効な書き込み可能なポインタが渡されることを期待します。
    pub unsafe fn features_bridge(
        m: bindings::Module,
        features: &mut Features,
        out: *mut *mut *mut i8,
    ) -> i32 {
        // Rust の Features から zsh 用の features 生構造体を構築
        // この際、内部の raw_builtins 等の Vec が更新され、ポインタが固定される
        let mut raw_f = features.as_zsh_features();

        // zsh 内部関数の featuresarray を呼び出す。
        // これにより、zsh が認識できる形式の文字列配列が作成され、out にセットされる。
        unsafe {
            let array_ptr = bindings::featuresarray(m, &mut raw_f);
            if !out.is_null() {
                *out = array_ptr;
            }
        }
        0
    }

    /// Zshが特定の機能を有効化/無効化（enables/disables）する際のブリッジ。
    ///
    /// # Safety
    /// Zshからの生ポインタ (`m`, `enables`) を扱うため、呼び出し元はZshのAPI規約に従う必要があります。
    /// 特に `enables` には有効な書き込み可能なポインタが渡されることを期待します。
    pub unsafe fn enables_bridge(
        m: bindings::Module,
        features: &mut Features,
        enables: *mut *mut i32,
    ) -> i32 {
        let mut raw_f = features.as_zsh_features();

        // zsh 内部関数の handlefeatures を呼び出す。
        // これにより、現在の有効/無効状態（ビットマップ等）が enables にセットされる。
        unsafe { bindings::handlefeatures(m, &mut raw_f, enables) }
    }
}

// Zshのbindingsから来る生ポインタを含む構造体が、Rustの並行処理モデルと互換性を持つようにします。
// `builtin`などの構造体はZshのメモリ管理下にあり、Rustの所有権システムから見るとSend/Syncの保証ができません。
// しかし、Zshモジュールのコンテキストでは、これらのポインタはシングルスレッドでアクセスされるか、
// Zsh自体がスレッドセーフな操作を保証するため、これらのトレイトを手動で実装します。
// これは、Rustのデータ競合に関する安全保証をバイパスするため、非常に注意深く行う必要があります。
/// `bindings::builtin` はZsh内部のポインタを含むため、RustのSend/Sync要件を自動的に満たしません。
/// ZshモジュールAPIのコンテキストでは、これらの構造体はZshによって管理され、
/// Rustの並行性モデルとは異なるライフサイクルとアクセスパターンを持つため、
/// `unsafe impl Send` および `unsafe impl Sync` を宣言して、`Features`構造体などが
/// `Mutex`のような並行性プリミティブ内で使用できるようにします。
/// これは、ZshのAPIがこれらのデータ構造へのアクセスにおいて、
/// 外部からのデータ競合を引き起こさないことを信頼して行われます。
unsafe impl Send for crate::bindings::builtin {}
unsafe impl Sync for crate::bindings::builtin {}
// 他の型も必要に応じて同様に追加 (例: `hookdef`, `conddef`, `paramdef`, `mathfunc` など、
// 生ポインタを含むかFFI経由で共有される可能性のある型)
