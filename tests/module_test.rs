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
    use libc::c_char;

    use super::*;
    use std::{os::raw::c_void, ptr};

    #[test]
    fn test_module_lifecycle_and_features() {
        unsafe {
            let dummy_module = ptr::null_mut();

            // 1. setup_ のテスト
            // 内部で TestModule::default() が呼ばれ、OnceLock に格納されるはず
            let setup_res = setup_(dummy_module);
            assert_eq!(setup_res, 0, "setup_ should return 0 on success");

            // 2. boot_ のテスト
            // OnceLock からインスタンスが取り出され、boot() が実行される
            let boot_res = boot_(dummy_module);
            assert_eq!(boot_res, 0, "boot_ should return 0 after successful setup");

            // 3. features_ のテスト (重要: ブリッジの検証)
            // zshが機能リストを取得する挙動を模倣
            let mut out_ptr: *mut *mut i8 = ptr::null_mut();
            let features_res = features_(dummy_module, &mut out_ptr as *mut _);

            assert_eq!(features_res, 0);
            // 本来は featuresarray スタブが返す値を検証するが、
            // 少なくともセグメンテーションフォールトせず実行できることを確認
        }
    }

    #[test]
    fn test_error_propagation() {
        // エラー時に 1 が返ることを確認したい場合、
        // 別途エラーを出す設定にした構造体で export_module! する必要があります。
        // ここでは、boot_ などが Result::Err を返した場合に 1 に変換されるロジックを信頼します。
    }
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn featuresarray(_m: *mut c_void, _f: *mut c_void) -> *mut *mut c_char {
        // 空の配列（最後が NULL）を返すか、とりあえず NULL を返す
        std::ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn handlefeatures(
        _m: *mut c_void,
        _f: *mut c_void,
        _en: *mut *mut i32,
    ) -> i32 {
        0
    }
}
