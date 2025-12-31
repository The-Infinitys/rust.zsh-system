use crate::bindings::{Builtin, Conddef, MathFunc, Paramdef, features};

/// zshの `features` 構造体を安全に構築・保持するためのラッパー
pub struct Features {
    builtins: Vec<Builtin>,
    conddefs: Vec<Conddef>,
    math_funcs: Vec<MathFunc>,
    param_defs: Vec<Paramdef>,
    n_abstract: i32,
}

// Featuresが定義されているファイル
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
        }
    }

    // 各機能を追加するメソッド（ビルダーパターン）
    pub fn add_builtin(mut self, builtin: Builtin) -> Self {
        self.builtins.push(builtin);
        self
    }

    pub fn add_param(mut self, param: Paramdef) -> Self {
        self.param_defs.push(param);
        self
    }

    // 最終的にzshへ渡す Raw 構造体を生成する
    // 注意: この構造体が生きている間だけ、返されたポインタは有効
    pub fn as_zsh_features(&self) -> features {
        features {
            bn_list: self.builtins.as_ptr() as Builtin,
            bn_size: self.builtins.len() as i32,
            cd_list: self.conddefs.as_ptr() as Conddef,
            cd_size: self.conddefs.len() as i32,
            mf_list: self.math_funcs.as_ptr() as MathFunc,
            mf_size: self.math_funcs.len() as i32,
            pd_list: self.param_defs.as_ptr() as Paramdef,
            pd_size: self.param_defs.len() as i32,
            n_abstract: self.n_abstract,
        }
    }
}
