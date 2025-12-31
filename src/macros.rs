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
        /// モジュール実装をカプセル化
        mod __zsh_module_impl {
            use super::*;
            use std::sync::{Mutex, OnceLock};
            use $crate::Features;

            pub struct ModuleContainer {
                pub instance: $module_struct,
                pub features_cache: Features,
            }

            /// 実体を保持するグローバルストレージ
            pub static MODULE_STORAGE: OnceLock<Mutex<ModuleContainer>> = OnceLock::new();

            /// 内部用：コンテナ全体へのアクセス
            pub fn with_container<R>(f: impl FnOnce(&mut ModuleContainer) -> R) -> R {
                let mutex = MODULE_STORAGE
                    .get()
                    .expect("Zsh module not initialized (setup_ not called)");
                let mut guard = mutex.lock().expect("Failed to lock module mutex");
                f(&mut *guard)
            }
        }

        /// 構造体名を通じて実体にアクセスするための拡張を実装
        impl $module_struct {
            /// Rustの他の場所から、このモジュールの実体（Runtimeなどを含む）にアクセスするための関数。
            ///
            /// # Example
            /// ```
            /// ZshInfinite::with_instance(|inst| {
            ///     inst.precmd()
            /// });
            /// ```
            pub fn with_instance<R>(f: impl FnOnce(&mut Self) -> R) -> R {
                __zsh_module_impl::with_container(|container| f(&mut container.instance))
            }
        }

        // --- Zsh エントリポイント ---

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn setup_(m: *mut i8) -> i32 {
            use std::sync::Mutex;
            let mut instance = <$module_struct as Default>::default();

            match instance.setup() {
                Ok(_) => {
                    let features_cache = instance.features();
                    let container = __zsh_module_impl::ModuleContainer {
                        instance,
                        features_cache,
                    };

                    if __zsh_module_impl::MODULE_STORAGE
                        .set(Mutex::new(container))
                        .is_err()
                    {
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

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn features_(m: *mut i8, out: *mut *mut *mut i8) -> i32 {
            __zsh_module_impl::with_container(|container| unsafe {
                $crate::__private_api::features_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    out,
                )
            })
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn enables_(m: *mut i8, enables: *mut *mut i32) -> i32 {
            __zsh_module_impl::with_container(|container| unsafe {
                $crate::__private_api::enables_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    enables,
                )
            })
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn boot_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_container(|c| match c.instance.boot() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: boot failed: {}", e);
                    1
                }
            })
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn cleanup_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_container(|c| match c.instance.cleanup() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: cleanup failed: {}", e);
                    1
                }
            })
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn finish_(m: *mut i8) -> i32 {
            __zsh_module_impl::with_container(|c| match c.instance.finish() {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("zsh-system: finish failed: {}", e);
                    1
                }
            })
        }
    };
}
