use crate::bindings;
use std::ffi::CString;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// zsh の `zalloc` / `zsfree` を使用するスマートポインタ
pub struct ZBox<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T> ZBox<T> {
    pub fn new(value: T) -> Self {
        unsafe {
            let ptr = bindings::zalloc(std::mem::size_of::<T>() as usize) as *mut T;
            let ptr = NonNull::new(ptr).expect("zsh: out of memory");
            std::ptr::write(ptr.as_ptr(), value);
            Self { ptr }
        }
    }
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("Attempted to wrap null pointer in ZBox"),
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T> Deref for ZBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for ZBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T: ?Sized> Drop for ZBox<T> {
    fn drop(&mut self) {
        unsafe {
            // zsh のメモリ解放
            bindings::zsfree(self.ptr.as_ptr() as *mut _);
        }
    }
}

pub struct ZString {
    inner: ZBox<i8>,
}

impl ZString {
    pub fn new(s: &str) -> Self {
        let c_str = CString::new(s).expect("...");
        unsafe {
            let ptr = bindings::ztrdup(c_str.as_ptr());
            Self {
                inner: ZBox::from_raw(ptr as *mut i8),
            }
        }
    }
    pub fn as_ptr(&self) -> *mut i8 {
        self.inner.as_ptr()
    }
}
