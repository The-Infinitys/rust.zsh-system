//! このモジュールは、Zshの数式関数（`mathfunc`）をRustで扱うための構造体を提供します。
//!
//! Zshのモジュールはカスタムの数式関数を登録でき、これによりZshスクリプト内で
//! `(( result = my_math_func(arg) ))` のような形式で利用可能な新しい関数を導入することができます。
use crate::ZString;
use crate::bindings::{NumMathFunc, StrMathFunc, mathfunc};

/// Zshの数式関数定義をカプセル化する構造体。
///
/// Zshの `mathfunc` 構造体に対応し、関数名、フラグ、数値ハンドラ、文字列ハンドラ、
/// 最小・最大引数を保持します。
pub struct Mathfunc {
    name: ZString,
    flags: i32,
    nfunc: NumMathFunc, // 数値引数を取るハンドラ関数ポインタ
    sfunc: StrMathFunc, // 文字列引数を取るハンドラ関数ポインタ
    min_args: i32,
    max_args: i32,
}

impl Mathfunc {
    /// `Mathfunc`インスタンスをZshの`mathfunc`構造体として表現します。
    ///
    /// この生構造体はZshのモジュールAPIに渡され、数式関数として登録されます。
    ///
    /// # Safety
    /// `std::mem::zeroed()` を使用して構造体をゼロ初期化し、
    /// 生ポインタの操作が含まれるため、この関数は`unsafe`です。
    pub fn as_raw(&self) -> mathfunc {
        mathfunc {
            name: self.name.as_ptr(),
            flags: self.flags,
            nfunc: self.nfunc,
            sfunc: self.sfunc,
            minargs: self.min_args,
            maxargs: self.max_args,
            ..unsafe { std::mem::zeroed() }
        }
    }
}
