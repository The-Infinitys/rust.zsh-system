use crate::ZString;
use crate::bindings::{CondHandler, conddef};

pub struct Conddef {
    name: ZString,
    flags: i32,
    handler: CondHandler,
    min: i32,
    max: i32,
    module: Option<ZString>,
}

impl Conddef {
    pub fn as_raw(&self) -> conddef {
        conddef {
            name: self.name.as_ptr(),
            flags: self.flags,
            handler: self.handler,
            min: self.min,
            max: self.max,
            module: self
                .module
                .as_ref()
                .map_or(std::ptr::null_mut(), |s| s.as_ptr()),
            ..unsafe { std::mem::zeroed() }
        }
    }
}
