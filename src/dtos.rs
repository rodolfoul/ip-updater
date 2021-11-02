use std::ffi::CStr;
use std::fmt;

#[repr(C)]
#[derive(Debug)]
pub enum Action {
    UNDEFINED,
    ADD,
    DEL,
}

#[repr(C)]
pub struct IfMessage {
    pub interface_name: [u8; 16],
    pub related_ip: [u8; 46],
    pub action: Action,
}


impl Default for IfMessage {
    fn default() -> IfMessage {
        IfMessage {
            interface_name: [0; 16],
            related_ip: [0; 46],
            action: Action::UNDEFINED,
        }
    }
}

impl fmt::Display for IfMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let interface_name = unsafe { CStr::from_ptr(&self.interface_name[0]) }
            .to_str()
            .unwrap();
        let related_ip = unsafe { CStr::from_ptr(&self.related_ip[0]) }
            .to_str()
            .unwrap();
        write!(
            f,
            "interface_name={}, relalted_ip={}, action={:?}",
            interface_name, related_ip, self.action
        )
    }
}