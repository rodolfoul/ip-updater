use super::dtos::IfMessage;
use nix::libc::NETLINK_ROUTE;
use nix::sys::socket::{bind, recv, socket, MsgFlags, NetlinkAddr, SockAddr};
use nix::unistd::close;
use std::os::unix::io::RawFd;
use std::process;

const RTMGRP_IPV6_IFADDR: u32 = 0x100;

extern "C" {
    fn parse_message(buf: *const u8, len: u32, message: &mut IfMessage);
}

pub struct MessageFetcher {
    fd: RawFd,
    buf: [u8; 1024],
}

impl MessageFetcher {
    pub fn new() -> Result<MessageFetcher, nix::Error> {
        let fd = match create_nl_socket() {
            Ok(fd) => fd,
            Err(e) => return Err(e),
        };

        let m = MessageFetcher { fd, buf: [0; 1024] };
        ctrlc::set_handler(move || close_and_exit(fd)).expect("Could not handle ctrl+c");

        Ok(m)
    }

    pub fn fetch_ip_change(&mut self) -> IfMessage {
        let read_len = match recv(self.fd, &mut self.buf, MsgFlags::empty()) {
            Err(e) => {
                println!("Error: {}", e);
                close(self.fd).expect("Failed closing fd while exiting");
                process::exit(1);
            }
            Ok(v) => v,
        };
        let mut message = IfMessage::default();
        unsafe {
            parse_message(self.buf.as_ptr(), read_len as u32, &mut message);
        }
        return message;
    }
}


fn close_and_exit(fd: RawFd) {
    close(fd).expect("Failed closing fd while exiting");
    process::exit(1);
}

fn create_nl_socket() -> Result<RawFd, nix::Error> {
    let fd = match socket(
        nix::sys::socket::AddressFamily::Netlink,
        nix::sys::socket::SockType::Raw,
        nix::sys::socket::SockFlag::empty(),
        NETLINK_ROUTE,
    ) {
        Ok(fd) => fd,
        Err(e) => return Err(e),
    };

    let addr = SockAddr::Netlink(NetlinkAddr::new(process::id(), RTMGRP_IPV6_IFADDR));
    match bind(fd, &addr) {
        Ok(_) => (),
        Err(e) => {
            close(fd).expect("Failed closing socket");
            return Err(e);
        }
    };
    return Ok(fd);
}
