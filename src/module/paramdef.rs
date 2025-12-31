//! このモジュールは、Zshのパラメータ定義（`paramdef`）をRustで扱うための構造体を提供します。
//!
//! Zshのモジュールはカスタムのパラメータを定義でき、これによりZshシェル内で
//! 新しい変数（スカラ、配列、連想配列）を導入し、その振る舞いを制御することができます。
use crate::ZString;
use crate::bindings::paramdef;

/// Zshのパラメータ定義をカプセル化する構造体。
///
/// Zshの `paramdef` 構造体に対応し、パラメータ名、フラグ、
/// 外部変数へのポインタ、および`get`/`set`/`unset`関数へのポインタ (`gsu`) を保持します。
pub struct Paramdef {
    name: ZString,
    flags: i32,
    var: *mut std::os::raw::c_void, // 外部変数のポインタ (例: i8, i32, char*, char**など)
    gsu: *const std::os::raw::c_void, // Get/Set/Unsetハンドラへのポインタ
}

impl Paramdef {
    /// `Paramdef`インスタンスをZshの`paramdef`構造体として表現します。
    ///
    /// この生構造体はZshのモジュールAPIに渡され、パラメータとして登録されます。
    ///
    /// # Safety
    /// `std::mem::zeroed()` を使用して構造体をゼロ初期化し、
    /// 生ポインタの操作が含まれるため、この関数は`unsafe`です。
    pub fn as_raw(&self) -> paramdef {
        paramdef {
            name: self.name.as_ptr(),
            flags: self.flags,
            var: self.var,
            gsu: self.gsu,
            ..unsafe { std::mem::zeroed() }
        }
    }
}
