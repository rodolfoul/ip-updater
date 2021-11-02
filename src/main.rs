mod dtos;
mod fetcher;

use chrono;
use dtos::{Action, IfMessage};
use eui48::{Eui64, MacAddress};
use fetcher::MessageFetcher;
use pnet::datalink;
use pnet::ipnetwork::IpNetwork;
use pnet::util::MacAddr;
use regex::Regex;
use std::io::{self, Write};
use std::net::Ipv6Addr;
use std::process;
use std::process::Command;
use std::str;
use std::str::FromStr;

fn main() {
    println!("My pid is {}", process::id());

    let internet_interface: &str = "usb0";
    let secondary_interface: &str = "eth0";
    let mut fetcher = MessageFetcher::new().unwrap();

    // loop {
    // let addr = fetcher.fetch_ip_change();
    // let usb0str = "usb0";
    // let mut usb0_arr: [u8; 16] = [0; 16];
    // usb0_arr[..usb0str.len()].clone_from_slice(usb0str.as_bytes());

    let ipstr = "2001::c6ff:fe30:f997";
    // let mut iparr: [u8; 46] = [0; 46];
    // iparr[..ipstr.len()].clone_from_slice(ipstr.as_bytes());

    // let addr_change_msg = IfMessage {
    //     interface_name: usb0_arr,
    //     related_ip: iparr,
    //     action: Action::ADD
    // };
    // let new_ip_str: &Ipv6Addr = Ipv6Addr::from_str(ipstr);
    let new_ip: Ipv6Addr = ipstr.parse().unwrap();
    let current_ips = get_stale_ip_addresses(&secondary_interface);
    remove_stale_ips(&secondary_interface, &current_ips);
    configure_new_ip_prefix_on(&secondary_interface, &new_ip.segments()[..4]);
    //Reconfigura prefixo da interface de roteamento, remove antigo /64, readiciona como /128
    //ifconfig usb0 del 2001:1284:f01c:5ea4:20e:c6ff:fe30:f34e/64
    //ifconfig usb0 add 2001:1284:f01c:5ea4:20e:c6ff:fe30:f34e/128
    
    let routes = get_routes();
    reconfigure_routes(&routes, internet_interface, ipstr, &new_ip.to_string());

    //replace string on files
    //restart services
}

fn reconfigure_routes(routes: &Vec<Route>, internet_if: &str, internet_ip: &str, secondary_ip: &str) {
// route -6 | grep -v lo | grep -ve fe80:: | grep -v ff00
// Remove rotas sem o prefixo novo
// remove rotas /64
// adiciona rota prefixo_novo/64 com o secondary_if
}

//Destination                    Next Hop                   Flag Met Ref Use If
//2001:1284:f01c:d89:20e:c6ff:fe30:f34e/128 [::]                       U    256 1     0 usb0
#[derive(Debug)]
struct Route {
    destination: String,
    next_hop: String,
    if_name: String,
}

fn get_routes() -> Vec<Route> {
    let route_output = Command::new("route").arg("-6").output().unwrap();
    let outout_str = String::from_utf8(route_output.stdout).unwrap();
    let mut lines_iter = outout_str.split("\n");
    lines_iter.next();
    lines_iter.next();
    let route_regex = Regex::new("^\\s*(\\S+)\\s+(\\S+).*?(\\w+)\\s*$").unwrap();
    return lines_iter
        .map(|s| route_regex.captures(s))
        .filter(|c| c.is_some())
        .map(|c| {
            let caps = c.unwrap();
            let r = Route {
                destination: caps.get(1).unwrap().as_str().to_string(),
                next_hop: caps.get(2).unwrap().as_str().to_string(),
                if_name: caps.get(3).unwrap().as_str().to_string(),
            };
            r
        })
        .collect();
}

fn remove_stale_ips(interface: &str, stale_ips: &Vec<IpNetwork>) {
    for ip in stale_ips {
        println!(
            "Removing ip/prefix {} from interface {}",
            ip.to_string(),
            interface
        );

        let ifconfig_output = Command::new("ifconfig")
            .arg(interface)
            .arg("del")
            .arg(ip.to_string())
            .output()
            .unwrap();

        io::stdout().write_all(&ifconfig_output.stdout).unwrap();
        io::stderr().write_all(&ifconfig_output.stderr).unwrap();

        let output_status = ifconfig_output.status.code().unwrap();
        if output_status != 0 {
            println!(
                "Failed while removing stale ips. ifconfig returned {}",
                output_status
            ); //TODO stop processing
        }
    }
}

fn configure_new_ip_prefix_on(interface_str: &str, head_ipv6: &[u16]) {
    let interfaces = datalink::interfaces();

    let mac_address: [u8; 6] = interfaces
        .iter()
        .find(|itf| interface_str.eq(&itf.name))
        .unwrap()
        .mac
        .unwrap()
        .into();
    let tail_ipv6 = &Ipv6Addr::from_str(
        &("::".to_owned()
            + &eui48::MacAddress::from_bytes(&mac_address)
                .unwrap()
                .to_interfaceid()),
    )
    .unwrap()
    .segments()[4..8];

    let new_ip_arr: [u16; 8] = [&head_ipv6[..4], tail_ipv6].concat().try_into().unwrap();
    let resulting_ip = Ipv6Addr::from(new_ip_arr).to_string() + "/64";

    println!(
        "Adding new ip/prefix {} to interface {}",
        resulting_ip, interface_str
    );
    let ifconfig_output = Command::new("ifconfig")
        .arg(interface_str)
        .arg("add")
        .arg(&resulting_ip)
        .output()
        .unwrap();

    io::stdout().write_all(&ifconfig_output.stdout).unwrap();
    io::stderr().write_all(&ifconfig_output.stderr).unwrap();

    let output_status = ifconfig_output.status.code().unwrap();
    if output_status != 0 {
        println!(
            "Failed while configuring new ip {} on {}. ifconfig returned {}",
            resulting_ip, interface_str, output_status
        ); //TODO stop processing
    }
}

fn get_stale_ip_addresses(interface_name: &str) -> Vec<IpNetwork> {
    let interfaces = datalink::interfaces();

    let interface = interfaces
        .iter()
        .find(|itf| interface_name.eq(&itf.name))
        .unwrap();
    let ips = &interface.ips;
    let filtered_ips = ips
        .iter()
        .filter(|ip| ip.is_ipv6())
        .filter(|ip| {
            let ip_str = ip.ip().to_string();

            !ip_str.starts_with("fe80")
        })
        .map(|ip| ip.to_owned())
        .collect();
    return filtered_ips;
}

fn print_changes(msg: IfMessage) {
    println!("{:?} - {}", chrono::offset::Local::now(), msg);
}
