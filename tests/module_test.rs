use zsh_system::{Features, ZshModule, export_module};

#[derive(Default)]
struct TestModule;

impl ZshModule for TestModule {
    fn features(&self) -> Features {
        Features::new() // テスト用に空のFeatures
    }
    fn setup(&mut self) -> i32 {
        123
    }
    fn boot(&mut self) -> i32 {
        456
    }
    fn cleanup(&mut self) -> i32 {
        789
    }
    fn finish(&mut self) -> i32 {
        0
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

#[test]
fn test_macro_symbols() {
    unsafe {
        // マクロによって生成された extern "C" 関数を直接叩いてみる
        // ※ bindings::Module のダミーとして 0 (null) を渡す
        assert_eq!(setup_(std::ptr::null_mut()), 123);
        assert_eq!(boot_(std::ptr::null_mut()), 456);
    }
}
