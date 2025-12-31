//! このモジュールは、Zshモジュールが提供する各種機能（ビルトイン、条件定義、数式関数、パラメータ定義）
//! を定義し、Zshの内部構造である `features` に変換するための安全なラッパーを提供します。
//!
//! `Features` 構造体は、これらの機能のRust表現を保持し、必要に応じてZshのC構造体に変換します。
use crate::bindings;
use crate::module::builtin::BuiltinHandler;
use crate::module::{Builtin, Conddef, Mathfunc, Paramdef};

/// Zshの `features` 構造体を安全に構築・保持するためのラッパー。
///
/// この構造体は、モジュールが提供するビルトインコマンド、条件定義、数式関数、
/// パラメータ定義を管理し、ZshのモジュールAPIに渡すための形式に変換します。
pub struct Features {
    builtins: Vec<Builtin>,
    conddefs: Vec<Conddef>,
    math_funcs: Vec<Mathfunc>,
    param_defs: Vec<Paramdef>,
    n_abstract: i32,

    /// Zshに渡すポインタの参照先を保持するためのキャッシュ。
    /// `as_zsh_features`内で作成される一時的な`Vec`が即座に解放されるのを防ぎ、
    /// Zshがアクセスする間、これらのデータが有効であることを保証します。
    raw_builtins: Vec<bindings::builtin>,
    raw_conddefs: Vec<bindings::conddef>,
    raw_mathfuncs: Vec<bindings::mathfunc>,
    raw_paramdefs: Vec<bindings::paramdef>,
}

/// `Features`はZshの内部ポインタへの参照を保持する可能性がありますが、
/// Zshモジュールのコンテキストでは、これらの参照はモジュールロード中に設定され、
/// その後のアクセスはZshによって同期されるか、シングルスレッドで行われることが期待されます。
/// したがって、`unsafe impl Send`と`unsafe impl Sync`を宣言することで、
/// `Features`が`Mutex`などの並行性プリミティブ内で安全に利用できるようにします。
unsafe impl Send for Features {}
unsafe impl Sync for Features {}

impl Default for Features {
    /// 新しい空の `Features` インスタンスを作成します。
    fn default() -> Self {
        Self::new()
    }
}

impl Features {
    /// 新しい空の `Features` インスタンスを作成します。
    pub fn new() -> Self {
        Self {
            builtins: Vec::new(),
            conddefs: Vec::new(),
            math_funcs: Vec::new(),
            param_defs: Vec::new(),
            n_abstract: 0,
            raw_builtins: Vec::new(),
            raw_conddefs: Vec::new(),
            raw_mathfuncs: Vec::new(),
            raw_paramdefs: Vec::new(),
        }
    }

    /// ビルトインコマンドを `Features` に追加します。
    ///
    /// ハンドラは内部的にグローバルディスパッチャに登録され、
    /// `Builtin`構造体が`features`リストに追加されます。
    pub fn add_builtin(mut self, name: &'static str, handler: BuiltinHandler) -> Self {
        use crate::module::builtin::{Builtin, register_handler};

        // 1. ハンドラをディスパッチャに登録
        register_handler(name, handler);

        // 2. ビルトイン定義を追加
        self.builtins.push(Builtin::new(name, handler));
        self
    }

    /// パラメータ定義を `Features` に追加します。
    pub fn add_param(mut self, param: Paramdef) -> Self {
        self.param_defs.push(param);
        self
    }

    /// `Features` インスタンスの内容をZshの `bindings::features` 構造体に変換します。
    ///
    /// このメソッドは、内部の `builtins` などの `Vec` から生のC構造体`Vec`を生成し、
    /// そのポインタを `bindings::features` に設定します。
    /// `raw_builtins` などのフィールドにこれらの`Vec`を保持することで、
    /// Zshがアクセスする間、メモリが解放されないようにします。
    pub fn as_zsh_features(&mut self) -> bindings::features {
        // 各 SafeWrapper から C の生構造体へ変換
        self.raw_builtins = self.builtins.iter().map(|b| b.as_raw()).collect();
        self.raw_conddefs = self.conddefs.iter().map(|c| c.as_raw()).collect();
        self.raw_mathfuncs = self.math_funcs.iter().map(|m| m.as_raw()).collect();
        self.raw_paramdefs = self.param_defs.iter().map(|p| p.as_raw()).collect();

        bindings::features {
            bn_list: self.raw_builtins.as_mut_ptr(),
            bn_size: self.raw_builtins.len() as i32,
            cd_list: self.raw_conddefs.as_mut_ptr(),
            cd_size: self.raw_conddefs.len() as i32,
            mf_list: self.raw_mathfuncs.as_mut_ptr(),
            mf_size: self.raw_mathfuncs.len() as i32,
            pd_list: self.raw_paramdefs.as_mut_ptr(),
            pd_size: self.raw_paramdefs.len() as i32,
            n_abstract: self.n_abstract,
        }
    }
}
