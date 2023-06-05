use colored::Colorize;
use std::net::SocketAddr;

const PAT: &str = r#"#######################################################################"#;
const TIT: &str = r#"@                      spvn - starting services                       @"#;
const SEP: &str = "@";
const SPC: &str = " ";

pub fn startup_message(pid: usize, addr: SocketAddr, tls: bool) {
    let mut addr_fmt = format!("{:?}", addr);
    if tls {
        addr_fmt = format!("https://{}", addr_fmt);
    } else {
        addr_fmt = format!("http://{}", addr_fmt);
    }

    let len_a = addr_fmt.len();
    let s = 34 - (len_a / 2);
    let pat_s = SPC.repeat(s);
    let fmt_addr = format!(
        "{}{}{}{}{}",
        SEP.blue(),
        pat_s,
        addr_fmt.blue(),
        pat_s,
        SEP.blue()
    );

    let inner = format!("process {}", pid);
    let pat_s = SPC.repeat(34 - (inner.len() / 2));
    let fmt_pid = format!(
        "{}{}{}{}{}",
        SEP.blue(),
        pat_s,
        inner.green(),
        pat_s,
        SEP.blue()
    );

    let fm = format!(
        "{}\n{}\n{}\n{}\n{}",
        PAT.black(),
        TIT.blue(),
        fmt_addr,
        fmt_pid,
        PAT.black()
    );
    println!("{}", fm);
}
