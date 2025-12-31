mod builtin;
mod conddef;
mod features;
mod mathfunc;
mod paramdef;
pub use builtin::Builtin;
pub use conddef::Conddef;
pub use features::Features;
pub use mathfunc::Mathfunc;
pub use paramdef::Paramdef;

use std::error::Error;

pub type ZshResult = Result<(), Box<dyn Error>>;

pub trait ZshModule {
    /// モジュールのセットアップ（初期ロード時）
    fn setup(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールの起動（機能が有効化される時）
    fn boot(&mut self) -> ZshResult {
        Ok(())
    }

    /// 終了処理（モジュールアンロードの直前）
    fn cleanup(&mut self) -> ZshResult {
        Ok(())
    }

    /// 最終破棄（メモリ解放など）
    fn finish(&mut self) -> ZshResult {
        Ok(())
    }

    /// モジュールが提供する機能の定義
    fn features(&self) -> Features;
}
