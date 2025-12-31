use crate::ZString;
use crate::bindings::{NumMathFunc, StrMathFunc, mathfunc};

pub struct Mathfunc {
    name: ZString,
    flags: i32,
    nfunc: NumMathFunc,
    sfunc: StrMathFunc,
    min_args: i32,
    max_args: i32,
}

impl Mathfunc {
    pub fn as_raw(&self) -> mathfunc {
        mathfunc {
            name: self.name.as_ptr(),
            flags: self.flags,
            nfunc: self.nfunc,
            sfunc: self.sfunc,
            minargs: self.min_args,
            maxargs: self.max_args,
            ..unsafe { std::mem::zeroed() }
        }
    }
}
