//! このモジュールは、ZshモジュールをRustで定義するためのマクロを提供します。
//!
//! [`export_module!`] マクロを使用することで、Zshがロード可能なモジュールの標準的なエントリポイント
//! (`setup_`, `features_`, `enables_`, `boot_`, `cleanup_`, `finish_`) を自動的に生成し、
//! Rustで実装されたモジュールのライフサイクルを管理します。
//!
//! 通常、ユーザーは自身のモジュール構造体に `ZshModule` トレイトを実装し、
//! その構造体をこのマクロに渡すことで、Zshモジュールとしてエクスポートします。
#[macro_export]
macro_rules! export_module {
    ($module_struct:ty) => {
        /// この定数は、マクロが展開されたことを示すマーカーとして機能します。
        #[allow(dead_code)]
        const ZSH_MODULE_EXPORTED: () = ();
        /// マクロ内部のシンボルが外部と衝突しないようにモジュールで包む
        mod __zsh_module_impl {
            use super::*;
            use std::sync::{Mutex, OnceLock};
            use $crate::Features;

            /// モジュールインスタンスとそのフィーチャーキャッシュを保持するコンテナ。
            pub struct ModuleContainer {
                pub instance: $module_struct,
                pub features_cache: Features,
            }

            /// モジュールインスタンスを保持するグローバルストレージ。
            /// `OnceLock` と `Mutex` を使用して、一度だけ初期化され、スレッドセーフにアクセスできるようにします。
            pub static MODULE_STORAGE: OnceLock<Mutex<ModuleContainer>> = OnceLock::new();

            /// モジュールインスタンスに安全にアクセスするためのヘルパー関数。
            /// `MODULE_STORAGE` の `Mutex` をロックし、モジュールコンテナへの可変参照を提供します。
            pub fn with_module<R>(f: impl FnOnce(&mut ModuleContainer) -> R) -> R {
                let mutex = MODULE_STORAGE
                    .get()
                    .expect("Zsh module not initialized (setup_ not called)");
                let mut guard = mutex.lock().expect("Failed to lock module mutex");
                f(&mut *guard)
            }
        }

        // --- Zsh エントリポイント (Extern "C" API) ---

        /// `setup_`: Zshがモジュールを最初にロードする際に呼び出されるエントリポイント。
        /// モジュールインスタンスの初期化と、`MODULE_STORAGE`への格納を行います。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn setup_(m: *mut i8) -> i32 {
            use std::sync::Mutex;
            // モジュールの初期化
            let mut instance = <$module_struct as Default>::default();

            match instance.setup() {
                Ok(_) => {
                    // 機能定義をキャッシュ
                    let features_cache = instance.features();
                    let container = __zsh_module_impl::ModuleContainer {
                        instance,
                        features_cache,
                    };

                    if __zsh_module_impl::MODULE_STORAGE
                        .set(Mutex::new(container))
                        .is_err()
                    {
                        eprintln!(
                            "zsh-system: Failed to initialize module storage (already initialized)"
                        );
                        return 1;
                    }
                    0
                }
                Err(e) => {
                    eprintln!("zsh-system: setup failed: {}", e);
                    1
                }
            }
        }

        /// `features_`: Zshがモジュールが提供する機能のリストを問い合わせる際に呼び出されるエントリポイント。
        /// `__private_api::features_bridge` を通じて、登録された機能をZshに渡します。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` および `out` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn features_(m: *mut i8, out: *mut *mut *mut i8) -> i32 {
            __zsh_module_impl::with_module(|container| unsafe {
                $crate::__private_api::features_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    out,
                )
            })
        }

        /// `enables_`: Zshがモジュール機能の有効/無効状態を変更する際に呼び出されるエントリポイント。
        /// `__private_api::enables_bridge` を通じて、Zshの要求を処理します。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` および `enables` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn enables_(m: *mut i8, enables: *mut *mut i32) -> i32 {
            __zsh_module_impl::with_module(|container| unsafe {
                $crate::__private_api::enables_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    enables,
                )
            })
        }

        /// `boot_`: Zshがモジュール機能を有効化した際に呼び出されるエントリポイント。
        /// モジュールに定義された `boot` メソッドを実行します。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn boot_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_module(|c| match c.instance.boot() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: boot failed: {}", e);
                    1
                }
            })
        }

        /// `cleanup_`: Zshがモジュール機能を無効化またはアンロードする際に呼び出されるエントリポイント。
        /// モジュールに定義された `cleanup` メソッドを実行します。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn cleanup_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_module(|c| match c.instance.cleanup() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: cleanup failed: {}", e);
                    1
                }
            })
        }

        /// `finish_`: Zshがモジュールを完全にアンロードし、リソースを解放する際に呼び出されるエントリポイント。
        /// モジュールに定義された `finish` メソッドを実行します。
        ///
        /// # Safety
        /// Zshによって呼び出される生ポインタ `m` を扱います。
        /// ZshのモジュールAPIの規約に従って安全に呼び出されることを前提とします。
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn finish_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_module(|c| match c.instance.finish() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: finish failed: {}", e);
                    1
                }
            })
        }
    };
}
