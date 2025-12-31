mod bindings;
mod macros;
mod module;
pub use crate::module::{Features, ZshModule};

#[doc(hidden)]
pub mod __private_api {
    use crate::bindings;
    use crate::module::Features;

    pub unsafe fn features_bridge(
        m: bindings::Module,
        feat: &mut Features,
        out: *mut *mut *mut i8,
    ) {
        let mut raw = feat.as_zsh_features();
        // 修正: unsafeブロックで囲む
        unsafe {
            *out = bindings::featuresarray(m, &mut raw);
        }
    }

    pub unsafe fn enables_bridge(
        m: bindings::Module,
        feat: &mut Features,
        enables: *mut *mut i32,
    ) -> i32 {
        let mut raw = feat.as_zsh_features();
        // 修正: unsafeブロックで囲む
        unsafe { bindings::handlefeatures(m, &mut raw, enables) }
    }
}

// エラー対策: 生ポインタを含むFeaturesをMutex/Staticで扱えるようにする
unsafe impl Send for crate::bindings::builtin {}
unsafe impl Sync for crate::bindings::builtin {}
// 他の型も必要に応じて同様に追加
