use crate::ZString;
use crate::bindings::{self, freearray, gethashnode2, param, paramtab, zlong, zsfree};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_double};

pub trait ZshParamType: Sized {}
impl ZshParamType for String {}
impl ZshParamType for i64 {}
impl ZshParamType for Vec<String> {}
impl ZshParamType for f64 {}

/// 特定の型に特化した高速ポインタハンドラ
pub struct ZshParamPtr<T: ZshParamType> {
    name: String,
    node: *mut param,
    _marker: PhantomData<T>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ZshType {
    Scalar,
    Array,
    Integer,
    Float,
}

impl<T: ZshParamType> ZshParamPtr<T> {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node: std::ptr::null_mut(),
            _marker: PhantomData,
        }
    }

    fn ensure_node(&mut self) -> Result<*mut param, &'static str> {
        unsafe {
            if !self.node.is_null() && !(*self.node).node.nam.is_null() {
                return Ok(self.node);
            }
            let c_name = CString::new(self.name.as_str()).map_err(|_| "Invalid name")?;
            let ptr = gethashnode2(paramtab, c_name.as_ptr() as *mut c_char);
            if ptr.is_null() {
                return Err("Parameter not found");
            }
            self.node = ptr as *mut param;
            Ok(self.node)
        }
    }
}

// --- String (Scalar) ---
impl ZshParamPtr<String> {
    pub fn get(&mut self) -> Result<String, &'static str> {
        let node = self.ensure_node()?;
        unsafe {
            let val_ptr = (*node).u.str_;
            if val_ptr.is_null() {
                Ok(String::new())
            } else {
                Ok(CStr::from_ptr(val_ptr).to_string_lossy().into_owned())
            }
        }
    }

    pub fn set(&mut self, value: &str) -> Result<(), &'static str> {
        let node = self.ensure_node()?;
        let z_str = ZString::new(value);
        let new_ptr = z_str.as_ptr();
        std::mem::forget(z_str);
        unsafe {
            let current_ptr = (*node).u.str_;
            if !current_ptr.is_null() {
                zsfree(current_ptr);
            }
            (*node).u.str_ = new_ptr;
        }
        Ok(())
    }
}

// --- i64 (Integer) ---
impl ZshParamPtr<i64> {
    pub fn get(&mut self) -> Result<i64, &'static str> {
        let node = self.ensure_node()?;
        unsafe { Ok((*node).u.val) }
    }
    pub fn set(&mut self, value: i64) -> Result<(), &'static str> {
        let node = self.ensure_node()?;
        unsafe {
            (*node).u.val = value as zlong;
        }
        Ok(())
    }
}

// --- f64 (Float) ---
impl ZshParamPtr<f64> {
    pub fn get(&mut self) -> Result<f64, &'static str> {
        let node = self.ensure_node()?;
        unsafe { Ok((*node).u.dval) }
    }
    pub fn set(&mut self, value: f64) -> Result<(), &'static str> {
        let node = self.ensure_node()?;
        unsafe {
            (*node).u.dval = value as c_double;
        }
        Ok(())
    }
}

// --- Vec<String> (Array) ---
impl ZshParamPtr<Vec<String>> {
    pub fn get(&mut self) -> Result<Vec<String>, &'static str> {
        let node = self.ensure_node()?;
        unsafe {
            let mut res = Vec::new();
            let mut curr = (*node).u.arr;
            if curr.is_null() {
                return Ok(res);
            }
            while !(*curr).is_null() {
                res.push(CStr::from_ptr(*curr).to_string_lossy().into_owned());
                curr = curr.add(1);
            }
            Ok(res)
        }
    }
    pub fn set(&mut self, values: Vec<&str>) -> Result<(), &'static str> {
        let node = self.ensure_node()?;
        unsafe {
            let count = values.len();
            let ptr_array = bindings::zalloc((count + 1) * std::mem::size_of::<*mut c_char>())
                as *mut *mut c_char;
            for (i, val) in values.into_iter().enumerate() {
                let z_val = ZString::new(val);
                *ptr_array.add(i) = z_val.as_ptr();
                std::mem::forget(z_val);
            }
            *ptr_array.add(count) = std::ptr::null_mut();
            let current_arr = (*node).u.arr;
            if !current_arr.is_null() {
                freearray(current_arr);
            }
            (*node).u.arr = ptr_array;
        }
        Ok(())
    }
}

// --- Any (Dynamic Access Handlers) ---

pub struct ZshAnyPtr {
    name: String,
    node: *mut param,
}

impl ZshAnyPtr {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            node: std::ptr::null_mut(),
        }
    }

    fn resolve(&mut self) -> Result<*mut param, &'static str> {
        unsafe {
            if !self.node.is_null() && !(*self.node).node.nam.is_null() {
                return Ok(self.node);
            }
            let c_name = CString::new(self.name.as_str()).map_err(|_| "Invalid name")?;
            let ptr = gethashnode2(paramtab, c_name.as_ptr() as *mut c_char);
            if ptr.is_null() {
                return Err("Parameter not found");
            }
            self.node = ptr as *mut param;
            Ok(self.node)
        }
    }

    pub fn get_type(&mut self) -> Result<ZshType, &'static str> {
        let node = self.resolve()?;
        let flags = unsafe { (*node).node.flags as u32 };
        if (flags & bindings::PM_ARRAY) != 0 {
            Ok(ZshType::Array)
        } else if (flags & bindings::PM_INTEGER) != 0 {
            Ok(ZshType::Integer)
        } else if (flags & bindings::PM_FFLOAT) != 0 {
            Ok(ZshType::Float)
        } else {
            Ok(ZshType::Scalar)
        }
    }

    /// あらゆる型から文字列として値を取得します
    pub fn get_as_string(&mut self) -> Result<String, &'static str> {
        match self.get_type()? {
            ZshType::Scalar => ZshParamPtr::<String>::new(&self.name).get(),
            ZshType::Integer => Ok(ZshParamPtr::<i64>::new(&self.name).get()?.to_string()),
            ZshType::Float => Ok(ZshParamPtr::<f64>::new(&self.name).get()?.to_string()),
            ZshType::Array => Ok(ZshParamPtr::<Vec<String>>::new(&self.name).get()?.join(" ")),
        }
    }

    /// Zsh側の型に合わせて文字列から適切に値をセットします
    pub fn set_from_string(&mut self, value: &str) -> Result<(), &'static str> {
        match self.get_type()? {
            ZshType::Scalar => ZshParamPtr::<String>::new(&self.name).set(value),
            ZshType::Integer => {
                let v = value.parse::<i64>().map_err(|_| "Not an integer")?;
                ZshParamPtr::<i64>::new(&self.name).set(v)
            }
            ZshType::Float => {
                let v = value.parse::<f64>().map_err(|_| "Not a float")?;
                ZshParamPtr::<f64>::new(&self.name).set(v)
            }
            ZshType::Array => {
                let v: Vec<&str> = value.split_whitespace().collect();
                ZshParamPtr::<Vec<String>>::new(&self.name).set(v)
            }
        }
    }
}
