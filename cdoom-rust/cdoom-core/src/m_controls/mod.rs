//! Shared binding loop for `m_controls`.
//!
//! C still owns the control globals on this branch; Rust owns the stable loop
//! that registers name/pointer pairs with the config system.

use std::os::raw::{c_char, c_int};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ControlBinding {
    pub name: *const c_char,
    pub location: *mut c_int,
}

pub type BindIntVariableFn = Option<unsafe extern "C" fn(*const c_char, *mut c_int)>;

pub unsafe fn bind_ints(
    bindings: *const ControlBinding,
    count: usize,
    bind_int_variable: BindIntVariableFn,
) {
    let Some(bind_int_variable) = bind_int_variable else {
        return;
    };

    if bindings.is_null() {
        return;
    }

    for i in 0..count {
        let binding = *bindings.add(i);
        bind_int_variable(binding.name, binding.location);
    }
}
