use crate::ZString;
use crate::bindings::paramdef;

pub struct Paramdef {
    name: ZString,
    flags: i32,
    var: *mut std::os::raw::c_void, // 外部変数のポインタ
    gsu: *const std::os::raw::c_void,
}

impl Paramdef {
    pub fn as_raw(&self) -> paramdef {
        paramdef {
            name: self.name.as_ptr(),
            flags: self.flags,
            var: self.var,
            gsu: self.gsu,
            ..unsafe { std::mem::zeroed() }
        }
    }
}
