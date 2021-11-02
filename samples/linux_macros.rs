use std::os::raw::{c_uint, c_ushort};
use std::ffi::c_void;
use core::{mem};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct rtattr {
    pub rta_len:  c_ushort,
    pub rta_type: c_ushort,
}

pub const RTA_ALIGNTO : c_uint = 4;

pub const fn RTA_ALIGN(len: c_uint) -> c_uint {
    (len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1)
}

pub unsafe fn RTA_DATA(rta: *mut rtattr) -> *mut c_void {
    (rta as usize + RTA_LENGTH(0) as usize) as *mut c_void
}

pub const fn RTA_LENGTH(len: c_uint) -> c_uint {
    RTA_ALIGN(mem::size_of::<rtattr>() as c_uint) + len
}

pub unsafe fn RTA_OK(rta: *const rtattr, len: c_uint) -> bool {
	len >= mem::size_of::<rtattr>() as c_uint
 && (*rta).rta_len >= mem::size_of::<rtattr>() as c_ushort
 && (*rta).rta_len as c_uint <= len
}