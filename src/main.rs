extern crate libc;

mod syscall;
mod trace;

use std::fs;
use trace::Trace;

fn main() {
    println!("Hello World!");
    let pid = get_zygote32_pid();
    let mut trace = Trace::new(pid);
    println!("zygote pid = {}", trace.pid());
    trace.trace();
}

fn get_zygote32_pid() -> i32 {
    let paths = fs::read_dir("/proc").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let path_str = path.display().to_string();
        let pid: Vec<&str> = path_str.split("/").collect();
        let pid = pid[2];
        if !pid.parse::<i32>().is_ok() {
            continue;
        }
        let file = path.join("cmdline");
        let cmdline = fs::read_to_string(file).unwrap();
        if cmdline.starts_with("zygote64") {
            continue;
        }
        if cmdline.starts_with("zygote") {
            return pid.parse().unwrap();
        }
    }
    return -1;
}
