//! このモジュールは、Zshの条件式定義（`conddef`）をRustで扱うための構造体を提供します。
//!
//! Zshのモジュールはカスタムの条件式を登録でき、これによりZshスクリプト内で
//! `if [[ ... ]]` の形式で利用可能な新しい条件を導入することができます。
use crate::ZString;
use crate::bindings::{CondHandler, conddef};

/// Zshの条件式定義をカプセル化する構造体。
///
/// Zshの `conddef` 構造体に対応し、条件式の名前、フラグ、ハンドラ関数、
/// 最小・最大引数、そして関連モジュールを保持します。
pub struct Conddef {
    name: ZString,
    flags: i32,
    handler: CondHandler, // ZshのC関数ポインタ
    min: i32,
    max: i32,
    module: Option<ZString>, // モジュール名 (オプション)
}

impl Conddef {
    /// `Conddef`インスタンスをZshの`conddef`構造体として表現します。
    ///
    /// この生構造体はZshのモジュールAPIに渡され、条件式として登録されます。
    ///
    /// # Safety
    /// `std::mem::zeroed()` を使用して構造体をゼロ初期化し、
    /// 生ポインタの操作が含まれるため、この関数は`unsafe`です。
    pub fn as_raw(&self) -> conddef {
        conddef {
            name: self.name.as_ptr(),
            flags: self.flags,
            handler: self.handler,
            min: self.min,
            max: self.max,
            module: self
                .module
                .as_ref()
                .map_or(std::ptr::null_mut(), |s| s.as_ptr()),
            ..unsafe { std::mem::zeroed() }
        }
    }
}
