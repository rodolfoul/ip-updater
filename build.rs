extern crate cc;
// use cc;

fn main() {
	cc::Build::new()
		.file("src/c/netlink_parser.c")
		// .define("FOO", Some("bar"))
		// .include("src")
		.compile("netlink_parser");
}
