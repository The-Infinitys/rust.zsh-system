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

pub trait ZshModule {
    /// モジュールのセットアップ（初期ロード時）
    fn setup(&mut self) -> i32 {
        0
    }

    /// モジュールの起動（機能が有効化される時）
    fn boot(&mut self) -> i32 {
        0
    }

    /// 終了処理（モジュールアンロードの直前）
    fn cleanup(&mut self) -> i32 {
        0
    }

    /// 最終破棄（メモリ解放など）
    fn finish(&mut self) -> i32 {
        0
    }

    /// モジュールが提供する機能（ビルトイン、パラメータ等）の定義
    /// ※ module_features 構造体に相当するデータを返す
    fn features(&self) -> Features;
}
