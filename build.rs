use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // 1. zsh-dev の存在チェック用のパス定義
    // Ubuntu/Debianでの標準的なパス
    let zsh_include_dir = "/usr/include/zsh";
    let zsh_config_dir = "/usr/lib/x86_64-linux-gnu/zsh/5.9/include";

    // 2. 明示的なエラーハンドリング
    if !Path::new(zsh_include_dir).exists() {
        panic!(
            "\n\n[zsh-system ERROR]: zsh-dev package not found!\n\
            Please install it using: sudo apt install zsh-dev\n\
            Expected path: {}\n",
            zsh_include_dir
        );
    }

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .derive_default(true)
        .blocklist_type("bool"); // Rustのboolと衝突するのを防ぐ

    // 3. Featuresに基づくパスの追加
    #[cfg(feature = "5-9")]
    let version = "5.9";
    builder = builder
        .clang_arg(format!("-I{}", zsh_include_dir))
        .clang_arg(format!("-I{}/{}", zsh_include_dir, version))
        .clang_arg(format!("-I{}/{}/zsh", zsh_include_dir, version))
        .clang_arg(format!("-I{}", zsh_config_dir));

    // バインディング生成
    let bindings = builder
        .generate()
        .expect("Unable to generate bindings. Check if clang is installed.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
