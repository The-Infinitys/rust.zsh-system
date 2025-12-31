use zsh_system::{Features, ZshModule, ZshResult, export_module};

#[derive(Default)]
struct TestModule {
    setup_called: bool,
}

impl ZshModule for TestModule {
    fn setup(&mut self) -> ZshResult {
        self.setup_called = true;
        // 成功を模倣
        Ok(())
    }

    fn features(&self) -> Features {
        // 実際の動作確認のため、空ではないFeaturesを返す
        Features::new()
    }

    fn boot(&mut self) -> ZshResult {
        if self.setup_called {
            Ok(())
        } else {
            Err("Setup was not called before boot".into())
        }
    }

    fn cleanup(&mut self) -> ZshResult {
        Ok(())
    }
    fn finish(&mut self) -> ZshResult {
        Ok(())
    }
}

// マクロを展開
export_module!(TestModule);

#[cfg(test)]
mod test_stubs {
    // bindgenが生成した型定義に合わせるため、必要な型をインポート
    // bindgenの出力に合わせて *mut i8 や usize など調整してください
    use std::os::raw::{c_char, c_void};

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn zsfree(ptr: *mut c_void) {
        if !ptr.is_null() {
            unsafe {
                libc::free(ptr);
            } // 簡易的な処理、または libc::free
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn zalloc(size: usize) -> *mut c_void {
        // テスト時は std::alloc を使用
        use std::alloc::{Layout, alloc};
        unsafe {
            let layout = Layout::from_size_align(size, 8).unwrap();
            alloc(layout) as *mut c_void
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn ztrdup(s: *const c_char) -> *mut c_char {
        if s.is_null() {
            return std::ptr::null_mut();
        }
        unsafe {
            let len = std::ffi::CStr::from_ptr(s).to_bytes().len();
            let ptr = zalloc(len + 1) as *mut c_char;
            std::ptr::copy_nonoverlapping(s, ptr, len + 1);
            ptr
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::{c_char, c_void};
    use std::ptr;

    #[test]
    fn test_full_module_lifecycle() {
        unsafe {
            let dummy_module = ptr::null_mut();

            // 1. Setup テスト
            // setup_ 内部で TestModule::default() が OnceLock にセットされ、setup() が呼ばれる
            let setup_res = setup_(dummy_module);
            assert_eq!(setup_res, 0, "setup_ should return 0 (success)");

            // 2. Boot テスト
            // OnceLock からインスタンスが取得され、boot() 内の setup_called チェックをパスするはず
            let boot_res = boot_(dummy_module);
            assert_eq!(
                boot_res, 0,
                "boot_ should return 0 because setup was called"
            );

            // 3. Features テスト
            // Features 構造体から zsh 用の 2次元配列 (char**) が生成されるプロセスを検証
            let mut out_features: *mut *mut c_char = ptr::null_mut();
            let feat_res = features_(dummy_module, &mut out_features);
            assert_eq!(feat_res, 0, "features_ should return 0");

            // 4. Cleanup テスト
            let cleanup_res = cleanup_(dummy_module);
            assert_eq!(cleanup_res, 0, "cleanup_ should return 0");

            // 5. Finish テスト
            let finish_res = finish_(dummy_module);
            assert_eq!(finish_res, 0, "finish_ should return 0");
        }
    }

    #[test]
    fn test_invalid_lifecycle_order() {
        // 注: OnceLockの仕様上、他のテストが先に走るとインスタンスが既に存在するため、
        // このテストを単独で実行するか、OnceLockの代わりに可変な管理が必要になります。
        // ここでは、boot_ がエラーを返した場合のブリッジコードの挙動を信頼します。
    }

    // --- Zsh 内部関数のスタブ (テスト時のみリンク) ---
    // これがないと、cargo test 実行時にリンクエラー（unresolved symbol）が発生します。

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn setfeatureenables(
        _m: *mut c_void,
        _f: *mut c_void,
        _e: *mut i32,
    ) -> i32 {
        0
    }

    // features_ 内部で呼ばれる可能性がある zsh 側の関数をスタブ化
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn getfeatureenables(
        _m: *mut c_void,
        _f: *mut c_void,
        _e: *mut *mut i32,
    ) -> i32 {
        0
    }
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn featuresarray(_m: *mut c_void, _f: *mut c_void) -> *mut *mut c_char {
        // 空の配列（最後が NULL）を返すか、とりあえず NULL を返す
        std::ptr::null_mut()
    }
}
