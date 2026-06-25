#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate axlog;
extern crate alloc;
extern crate axruntime;

use alloc::{format, string::ToString};

mod entry;
mod mm;
mod syscall;

const LOGO: &str = r#"
  SWTC Operating System
  LoongArch evaluation kernel
"#;

fn print_logo() {
    ax_println!("{}", LOGO);
}

#[unsafe(no_mangle)]
fn main() {
    print_logo();
    xprocess::Process::new_init(axtask::current().id().as_u64() as _).build();
    xcore::fs::vfs::init_root().expect("Failed to mount vfs");
    xcore::fs::fd::init_stdio().expect("Failed to init stdio");

    let envs = [format!("ARCH={}", option_env!("ARCH").unwrap_or("unknown"))];

    #[cfg(feature = "init-test")]
    let init = include_str!("test.sh");
    #[cfg(not(feature = "init-test"))]
    let init = include_str!("init.sh");

    info!("Running init script");
    let args = ["/musl/busybox", "sh", "-c", init]
        .map(|s| s.to_string())
        .to_vec();
    let exit_code = entry::run_user_app(&args, &envs);
    info!("Init script exited with code: {:?}", exit_code);
    axhal::misc::terminate();
}
