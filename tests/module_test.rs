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

#[test]
fn test_macro_symbols() {
    unsafe {
        // マクロによって生成された extern "C" 関数を直接叩いてみる
        // ※ bindings::Module のダミーとして 0 (null) を渡す
        assert_eq!(setup_(std::ptr::null_mut()), 123);
        assert_eq!(boot_(std::ptr::null_mut()), 456);
    }
}
