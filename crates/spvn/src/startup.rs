use colored::Colorize;
use std::net::SocketAddr;

const pat: &str = r#"#######################################################################"#;
const tit: &str = r#"@                      spvn - starting services                       @"#;
const sep: &str = "@";
const spc: &str = " ";

pub fn startup_message(pid: usize, addr: SocketAddr, tls: bool) {
    let mut addr_fmt = format!("{:?}", addr);
    if tls {
        addr_fmt = format!("https://{}", addr_fmt);
    } else {
        addr_fmt = format!("http://{}", addr_fmt);
    }

    let len_a = addr_fmt.len();
    let s = 34 - (len_a / 2);
    let pat_s = spc.repeat(s);
    let fmt_addr = format!(
        "{}{}{}{}{}",
        sep.blue(),
        pat_s,
        addr_fmt.blue(),
        pat_s,
        sep.blue()
    );

    let inner = format!("process {}", pid);
    let pat_s = spc.repeat(34 - (inner.len() / 2));
    let fmt_pid = format!(
        "{}{}{}{}{}",
        sep.blue(),
        pat_s,
        inner.green(),
        pat_s,
        sep.blue()
    );

    let fm = format!(
        "{}\n{}\n{}\n{}\n{}",
        pat.black(),
        tit.blue(),
        fmt_addr,
        fmt_pid,
        pat.black()
    );
    println!("{}", fm);
}
