//! このモジュールは、ZshモジュールをRustで実装するための主要なトレイトと型を定義します。
//!
//! Zshモジュールとして機能するために必要なライフサイクルメソッドと、
//! 提供する機能を定義するためのインターフェースを提供します。
mod builtin;
mod conddef;
mod features;
mod hook;
mod mathfunc;
mod paramdef;
pub use builtin::*;
pub use conddef::*;
pub use features::*;
pub use hook::*;
pub use mathfunc::*;
pub use paramdef::*;

use std::error::Error;

/// Zshモジュール操作の結果を示す型エイリアス。
///
/// 成功時には `()` を返し、失敗時にはトレイトオブジェクト `Box<dyn Error>` を返します。
pub type ZshResult = Result<(), Box<dyn Error>>;

/// Zshモジュールのライフサイクルと機能定義を抽象化するトレイト。
///
/// このトレイトを実装することで、Rustの構造体をZshモジュールとして機能させることができます。
pub trait ZshModule {
    /// モジュールのセットアップ処理。
    /// Zshがモジュールを最初にロードする際に一度だけ呼び出されます。
    /// ここで、グローバルな初期化やリソースの確保を行います。
    fn setup(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールの起動処理。
    /// Zshがモジュール内の機能を有効にする際に呼び出されます。
    /// 例えば、ビルトインコマンドが初めて使用される前などに実行されることがあります。
    fn boot(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールのクリーンアップ処理。
    /// Zshがモジュールをアンロードする際、または機能を無効にする際に呼び出されます。
    /// `setup`や`boot`で確保したリソースの解放を行います。
    fn cleanup(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールの最終破棄処理。
    /// `cleanup`の後に呼び出され、モジュールが完全にメモリから解放される直前に行われるべき処理を定義します。
    fn finish(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールが提供するZsh機能の定義。
    ///
    /// このメソッドは、モジュールがどのようなビルトインコマンド、条件定義、数式関数、
    /// パラメータ定義などをZshに提供するかを `Features` 構造体として返します。
    fn features(&self) -> Features;
}
