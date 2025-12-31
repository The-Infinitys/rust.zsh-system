//! `zsh-system` のビルドスクリプト。
//!
//! `bindgen` を使用して Zsh のヘッダーファイルから Rust の FFI (Foreign Function Interface) バインディングを生成します。
//! これにより、Rust コードから Zsh の C API を安全に呼び出すことができるようになります。
//!
//! Zsh の開発用ヘッダーファイルが特定のパスに存在することを前提とし、
//! 見つからない場合はエラーを発生させます。
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let wrapper_path = Path::new("lib/wrapper.h");
    if !wrapper_path.exists() {
        panic!("couldn't found wrapper file!");
    }
    println!("cargo:rerun-if-changed={}", wrapper_path.display());

    // Zsh開発パッケージの存在チェックとパス定義
    // Ubuntu/Debianなどの標準的なパスを想定
    let zsh_include_dir = "/usr/include/zsh";
    let zsh_config_dir = "/usr/lib/x86_64-linux-gnu/zsh/5.9/include";

    // 明示的なエラーハンドリング: zsh-dev パッケージがインストールされているかを確認
    if !Path::new(zsh_include_dir).exists() {
        panic!(
            "\n\n[zsh-system ERROR]: zsh-dev package not found!\n\
            Please install it using: sudo apt install zsh-dev\n\
            Expected path: {}\n",
            zsh_include_dir
        );
    }

    let mut builder = bindgen::Builder::default()
        .header(wrapper_path.display().to_string())
        .derive_default(true)
        .blocklist_type("bool"); // Rustのboolと衝突するのを防ぐ

    // Featuresに基づくパスの追加
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
