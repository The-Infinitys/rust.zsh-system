use zsh_system::{Features, ZshModule, ZshResult, export_module};

#[derive(Default)]
struct TestModule {
    setup_called: bool,
}

impl ZshModule for TestModule {
    fn setup(&mut self) -> ZshResult {
        self.setup_called = true;
        Ok(())
    }
    fn features(&self) -> Features {
        Features::new()
    }
    fn boot(&mut self) -> ZshResult {
        if self.setup_called {
            Ok(())
        } else {
            Err("Setup not called".into())
        }
    }
    fn cleanup(&mut self) -> ZshResult {
        Ok(())
    }
    fn finish(&mut self) -> ZshResult {
        Ok(())
    }
}

export_module!(TestModule);

// --- テスト専用スタブ (libzsh.so がない環境用) ---
#[cfg(test)]
mod test_stubs {
    use std::os::raw::{c_char, c_void};

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn zsfree(ptr: *mut c_void) {
        if !ptr.is_null() {
            unsafe { libc::free(ptr) }
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn zalloc(size: usize) -> *mut c_void {
        unsafe { libc::malloc(size) }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn ztrdup(s: *const c_char) -> *mut c_char {
        if s.is_null() {
            return std::ptr::null_mut();
        }
        unsafe { libc::strdup(s) }
    }

    // zshの機能をエミュレートするための空関数
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn setfeatureenables(
        _m: *mut c_void,
        _f: *mut c_void,
        _e: *mut i32,
    ) -> i32 {
        0
    }
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn getfeatureenables(
        _m: *mut c_void,
        _f: *mut c_void,
        _e: *mut *mut i32,
    ) -> i32 {
        0
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::os::raw::{c_char, c_void};
    use std::ptr;
    use std::sync::Mutex;
    use zsh_system::{ZshHookFn, bindings, zsh_hook_handler};

    struct TestData {
        counter: i32,
    }

    zsh_hook_handler!(test_hook_handler, context, {
        if let Some(data) = unsafe { context.data::<TestData>() } {
            data.counter += 1;
        }
        0
    });

    // 以前のエラーを回避するため、内部構造体をラップして Sync を付与
    struct SyncHookDef(bindings::hookdef);
    unsafe impl Send for SyncHookDef {}
    unsafe impl Sync for SyncHookDef {}

    static DUMMY_HOOK: Mutex<Option<SyncHookDef>> = Mutex::new(None);
    static mut REGISTERED_FUNCS: Vec<ZshHookFn> = Vec::new();

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn gethookdef(name: *mut c_char) -> *mut bindings::hookdef {
        let name_str = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap();
        if name_str == "test_event" {
            let mut guard = DUMMY_HOOK.lock().unwrap();
            if guard.is_none() {
                let mut h: bindings::hookdef = unsafe { std::mem::zeroed() };
                h.name = name;
                *guard = Some(SyncHookDef(h));
            }
            // static mut への直接参照 (&mut) を避け、addr_of_mut! 等からポインタを取得
            return &mut guard.as_mut().unwrap().0 as *mut bindings::hookdef;
        }
        ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn addhookfunc(_n: *mut c_char, f: ZshHookFn) -> i32 {
        // static mut への直接参照を避け、addr_of_mut! でポインタとして扱う
        unsafe {
            let reg_ptr = std::ptr::addr_of_mut!(REGISTERED_FUNCS);
            (*reg_ptr).push(f);
        }
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn deletehookfunc(_n: *mut c_char, f: ZshHookFn) -> i32 {
        unsafe {
            let reg_ptr = std::ptr::addr_of_mut!(REGISTERED_FUNCS);
            // 関数ポインタの比較には std::ptr::fn_addr_eq を使用 (unpredictable_function_pointer_comparisons 対応)
            (*reg_ptr).retain(|&x| !std::ptr::fn_addr_eq(x, f));
        }
        0
    }
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn runhookdef(h: *mut bindings::hookdef, data: *mut c_void) {
        unsafe {
            test_hook_handler(h, data);
        }
    }

    #[test]
    fn test_module_lifecycle_and_hooks() {
        unsafe {
            let dummy_module = ptr::null_mut();
            assert_eq!(setup_(dummy_module), 0);
            assert_eq!(boot_(dummy_module), 0);

            // Hookテスト
            let mut my_data = TestData { counter: 10 };
            let hdef = gethookdef(b"test_event\0".as_ptr() as *mut c_char);
            runhookdef(hdef, &mut my_data as *mut _ as *mut c_void);
            assert_eq!(my_data.counter, 11);

            assert_eq!(cleanup_(dummy_module), 0);
            assert_eq!(finish_(dummy_module), 0);
        }
    }
}
