//! このモジュールは、Zshのメモリ管理関数 (`zalloc`, `zsfree`, `ztrdup`) を利用して
//! ヒープメモリを管理するためのRustスマートポインタを提供します。
//!
//! これにより、Zshの内部APIと安全にメモリを共有し、Rustの所有権システムと統合することができます。
use crate::bindings;
use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Zshのメモリ管理 (`zalloc` / `zsfree`) を使用するスマートポインタ。
///
/// Rustの `Box` に似ていますが、メモリの確保と解放にZshのAPIを使用します。
/// これにより、ZshとRust間でメモリ所有権を安全に受け渡すことができます。
pub struct ZBox<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T> ZBox<T> {
    /// Zshの `zalloc` を使用して新しいメモリを確保し、`value` を初期化して `ZBox` を作成します。
    ///
    /// # Panics
    /// メモリ確保に失敗した場合（`zalloc`がNULLを返した場合）パニックします。
    pub fn new(value: T) -> Self {
        unsafe {
            // Zshのzallocを使ってメモリを確保
            let ptr = bindings::zalloc(std::mem::size_of::<T>()) as *mut T;
            let ptr = NonNull::new(ptr).expect("zsh: out of memory");
            // Rustの値を確保したメモリに書き込む (所有権を移動)
            std::ptr::write(ptr.as_ptr(), value);
            Self { ptr }
        }
    }

    /// 生ポインタを `ZBox` でラップします。
    ///
    /// # Safety
    /// `ptr` はZshの `zalloc` によって確保され、まだ解放されていない有効なポインタでなければなりません。
    /// `ptr` がNULLの場合、パニックします。
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("Attempted to wrap null pointer in ZBox"),
        }
    }

    /// `ZBox`が指す生ポインタを返します。
    ///
    /// 返されたポインタは `ZBox` のライフタイムの間有効です。
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Deref for ZBox<T> {
    type Target = T;
    /// `ZBox` の参照を、それが指す値の参照にデリファレンスします。
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for ZBox<T> {
    /// `ZBox` の可変参照を、それが指す値の可変参照にデリファレンスします。
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for ZBox<T> {
    /// `ZBox` がスコープを外れるときに、Zshの `zsfree` を使用してメモリを解放します。
    ///
    /// # Safety
    /// `zsfree` はCの関数であり、解放されるメモリが `zalloc` で確保されたものであることを前提とします。
    /// 不適切なポインタを解放しようとすると未定義動作を引き起こす可能性があります。
    fn drop(&mut self) {
        unsafe {
            // Zsh のメモリ解放
            bindings::zsfree(self.ptr.as_ptr() as *mut _);
        }
    }
}

/// Zshの文字列 (`char*`) を管理するためのスマートポインタ。
///
/// Zshの `ztrdup` を使用して文字列を複製し、`zsfree` で解放します。
/// Rustの `String` や `&str` とZshの文字列フォーマット間の変換を安全に行うために使用されます。
pub struct ZString {
    inner: ZBox<i8>,
}

impl ZString {
    /// Rustの文字列スライス `s` から新しい `ZString` を作成します。
    ///
    /// Zshの `ztrdup` を使用して文字列を複製し、その所有権を `ZString` が管理します。
    ///
    /// # Panics
    /// 内部的に`CString::new`が失敗した場合（文字列にNULLバイトが含まれる場合など）、パニックします。
    pub fn new(s: &str) -> Self {
        let c_str = CString::new(s).expect("ZString::new failed: input string contains null bytes");
        unsafe {
            // Zshのztrdupを使って文字列を複製
            let ptr = bindings::ztrdup(c_str.as_ptr());
            Self {
                inner: ZBox::from_raw(ptr),
            }
        }
    }

    /// `ZString`が指す生ポインタ (`*mut i8`) を返します。
    ///
    /// 返されたポインタは `ZString` のライフタイムの間有効です。
    pub fn as_ptr(&self) -> *mut i8 {
        self.inner.as_ptr()
    }
}
