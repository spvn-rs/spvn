use colored::Colorize;
use std::net::SocketAddr;

const pat: &str = r#"######################################################################"#;
const tit: &str = r#"@                      spvn - starting services                      @"#;
const sep: &str = "@";
const spc: &str = " ";

pub fn startup_message(addr: SocketAddr, tls: bool) {
    let mut addr_fmt = format!("{:?}", addr);
    if tls {
        addr_fmt = format!("https://{}", addr_fmt);
    } else {
        addr_fmt = format!("http://{}", addr_fmt);
    }

    let len_a = addr_fmt.len();
    let s = 34 - (len_a / 2);
    let pat_s = spc.repeat(s);
    let formatted = format!("{}{}{}{}{}", sep, pat_s, addr_fmt, pat_s, sep);
    let fm = format!(
        "{}\n{}\n{}\n{}",
        pat.black(),
        tit.blue(),
        formatted.blue(),
        pat.black()
    );
    println!("{}", fm);
}
