#[macro_export]
macro_rules! export_module {
    ($module_struct:ty) => {
        #[allow(dead_code)]
        const ZSH_MODULE_EXPORTED: () = ();
        /// マクロ内部のシンボルが外部と衝突しないようにモジュールで包む
        mod __zsh_module_impl {
            use super::*;
            use std::sync::{Mutex, OnceLock};
            use $crate::Features;

            pub struct ModuleContainer {
                pub instance: $module_struct,
                pub features_cache: Features,
            }

            /// モジュールインスタンスを保持するグローバルストレージ
            pub static MODULE_STORAGE: OnceLock<Mutex<ModuleContainer>> = OnceLock::new();

            /// インスタンスに安全にアクセスするためのヘルパー
            pub fn with_module<R>(f: impl FnOnce(&mut ModuleContainer) -> R) -> R {
                let mutex = MODULE_STORAGE
                    .get()
                    .expect("Zsh module not initialized (setup_ not called)");
                let mut guard = mutex.lock().expect("Failed to lock module mutex");
                f(&mut *guard)
            }
        }

        // --- Zsh エントリポイント (Extern "C" API) ---

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
