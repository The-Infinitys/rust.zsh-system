#[macro_export]
macro_rules! export_module {
    ($module_struct:ty) => {
        use std::sync::{Mutex, OnceLock};

        struct ModuleContainer {
            instance: $module_struct,
            features_cache: $crate::Features,
        }

        // Module型をマクロ内で直接参照せず、ポインタとして扱う
        static MODULE_STORAGE: OnceLock<Mutex<ModuleContainer>> = OnceLock::new();

        fn with_module<R>(f: impl FnOnce(&mut ModuleContainer) -> R) -> R {
            let mutex = MODULE_STORAGE.get().expect("Module not initialized");
            let mut guard = mutex.lock().expect("Failed to lock module");
            f(&mut *guard)
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn setup_(m: *mut i8) -> i32 {
            let mut instance = <$module_struct as Default>::default();
            let res = instance.setup();
            // 修正: トレイトのメソッド名を確認。エラーに基づき `features` に変更
            let features_cache = instance.features();

            let container = ModuleContainer {
                instance,
                features_cache,
            };
            let _ = MODULE_STORAGE.set(Mutex::new(container));
            res
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn features_(m: *mut i8, out: *mut *mut *mut i8) -> i32 {
            with_module(|container| unsafe {
                $crate::__private_api::features_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    out,
                );
            });
            0
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn enables_(m: *mut i8, enables: *mut *mut i32) -> i32 {
            with_module(|container| unsafe {
                $crate::__private_api::enables_bridge(
                    m as *mut _,
                    &mut container.features_cache,
                    enables,
                )
            })
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn boot_(m: *mut i8) -> i32 {
            with_module(|c| c.instance.boot())
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn cleanup_(m: *mut i8) -> i32 {
            with_module(|c| c.instance.cleanup())
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn finish_(m: *mut i8) -> i32 {
            with_module(|c| c.instance.finish())
        }
    };
}
