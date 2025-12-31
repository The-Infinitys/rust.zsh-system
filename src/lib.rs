mod macros;
mod module;
mod zalloc;
pub use crate::module::*;
pub use zalloc::*;
#[doc(hidden)]
pub mod bindings;

#[doc(hidden)]
pub mod __private_api {
    use crate::Features;
    use crate::bindings;

    /// zsh がモジュールから提供される「機能の名前リスト」を取得するためのブリッジ
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

    /// zsh が特定の機能を有効化/無効化（enables/disables）する際のブリッジ
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

// エラー対策: 生ポインタを含むFeaturesをMutex/Staticで扱えるようにする
unsafe impl Send for crate::bindings::builtin {}
unsafe impl Sync for crate::bindings::builtin {}
// 他の型も必要に応じて同様に追加
