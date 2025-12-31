use crate::ZString;
use crate::bindings::{HandlerFunc, builtin, hashnode};

pub struct Builtin {
    name: ZString,
    handler: HandlerFunc,
    min_args: i32,
    max_args: i32,
    optstr: Option<ZString>,
}

impl Builtin {
    pub fn as_raw(&self) -> builtin {
        builtin {
            // zshのhashnodeのnamフィールドにポインタを渡す
            node: hashnode {
                nam: self.name.as_ptr(),
                ..unsafe { std::mem::zeroed() }
            },
            handlerfunc: self.handler,
            minargs: self.min_args,
            maxargs: self.max_args,
            optstr: self
                .optstr
                .as_ref()
                .map_or(std::ptr::null_mut(), |s| s.as_ptr()),
            ..unsafe { std::mem::zeroed() }
        }
    }
}
