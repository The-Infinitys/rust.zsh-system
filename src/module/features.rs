use crate::bindings;
use crate::module::builtin::BuiltinHandler;
use crate::module::{Builtin, Conddef, Mathfunc, Paramdef};

/// zshの `features` 構造体を安全に構築・保持するためのラッパー
pub struct Features {
    builtins: Vec<Builtin>,
    conddefs: Vec<Conddef>,
    math_funcs: Vec<Mathfunc>,
    param_defs: Vec<Paramdef>,
    n_abstract: i32,

    // Zshに渡すポインタの参照先を保持するためのキャッシュ
    // これがないと、as_zsh_features 内で作った一時的な Vec は即座に解放されてしまいます
    raw_builtins: Vec<bindings::builtin>,
    raw_conddefs: Vec<bindings::conddef>,
    raw_mathfuncs: Vec<bindings::mathfunc>,
    raw_paramdefs: Vec<bindings::paramdef>,
}

unsafe impl Send for Features {}
unsafe impl Sync for Features {}

impl Features {
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

    pub fn add_builtin(mut self, name: &'static str, handler: BuiltinHandler) -> Self {
        use crate::module::builtin::{Builtin, register_handler};

        // 1. ハンドラをディスパッチャに登録
        register_handler(name, handler);

        // 2. ビルトイン定義を追加
        self.builtins.push(Builtin::new(name, handler));
        self
    }

    pub fn add_param(mut self, param: Paramdef) -> Self {
        self.param_defs.push(param);
        self
    }
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
