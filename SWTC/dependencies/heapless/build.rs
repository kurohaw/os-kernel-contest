#![deny(warnings)]

use std::{env, error::Error};

use rustc_version::Channel;

fn main() -> Result<(), Box<dyn Error>> {
    // Cargo only caps lints for registry dependencies. This vendored crate is
    // patched in as a path dependency, so its `deny(warnings)` also applies to
    // rustc's `unexpected_cfgs` lint. Declare every cfg emitted or consumed by
    // this build script before selecting the target-specific ones below.
    for cfg in [
        "armv6m",
        "armv7a",
        "armv7m",
        "armv7r",
        "armv8m_base",
        "armv8m_main",
        "cas_atomic_polyfill",
        "full_atomic_polyfill",
        "has_atomics",
        "has_cas",
        "unstable_channel",
    ] {
        println!("cargo:rustc-check-cfg=cfg({cfg})");
    }

    let target = env::var("TARGET")?;

    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=armv6m");
    } else if target.starts_with("thumbv7m-") {
        println!("cargo:rustc-cfg=armv7m");
    } else if target.starts_with("thumbv7em-") {
        println!("cargo:rustc-cfg=armv7m");
    } else if target.starts_with("armv7r-") | target.starts_with("armebv7r-") {
        println!("cargo:rustc-cfg=armv7r");
    } else if target.starts_with("thumbv8m.base") {
        println!("cargo:rustc-cfg=armv8m_base");
    } else if target.starts_with("thumbv8m.main") {
        println!("cargo:rustc-cfg=armv8m_main");
    } else if target.starts_with("armv7-") | target.starts_with("armv7a-") {
        println!("cargo:rustc-cfg=armv7a");
    }

    let is_avr = env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("avr");

    // built-in targets with no atomic / CAS support as of nightly-2022-01-13
    // AND not supported by the atomic-polyfill crate
    // see the `no-atomics.sh` / `no-cas.sh` script sitting next to this file
    if is_avr {
        // lacks cas
    } else {
        match &target[..] {
            "avr-unknown-gnu-atmega328"
                | "bpfeb-unknown-none"
                | "bpfel-unknown-none"
                | "msp430-none-elf"
                // | "riscv32i-unknown-none-elf"    // supported by atomic-polyfill
                // | "riscv32imc-unknown-none-elf"  // supported by atomic-polyfill
                | "thumbv4t-none-eabi"
                // | "thumbv6m-none-eabi"           // supported by atomic-polyfill
                // | "xtensa-esp32s2-none-elf"      // supported by atomic-polyfill
                => {}

            _ => {
                println!("cargo:rustc-cfg=has_cas");
            }
        }
    };

    if is_avr {
        // lacks atomics
    } else {
        match &target[..] {
        "msp430-none-elf"
        // | "riscv32i-unknown-none-elf"    // supported by atomic-polyfill
        // | "riscv32imc-unknown-none-elf"  // supported by atomic-polyfill
        // | "xtensa-esp32s2-none-elf"  // supported by atomic-polyfill
        => {}

        _ => {
            println!("cargo:rustc-cfg=has_atomics");
        }
    }
    };

    // Let the code know if it should use atomic-polyfill or not, and what aspects
    // of polyfill it requires
    if is_avr {
        println!("cargo:rustc-cfg=full_atomic_polyfill");
        println!("cargo:rustc-cfg=cas_atomic_polyfill");
    } else {
        match &target[..] {
            "riscv32i-unknown-none-elf" | "riscv32imc-unknown-none-elf" => {
                println!("cargo:rustc-cfg=full_atomic_polyfill");
                println!("cargo:rustc-cfg=cas_atomic_polyfill");
            }

            "thumbv6m-none-eabi" | "xtensa-esp32s2-none-elf" => {
                println!("cargo:rustc-cfg=cas_atomic_polyfill");
            }
            _ => {}
        }
    }

    if !matches!(
        rustc_version::version_meta().unwrap().channel,
        Channel::Stable | Channel::Beta
    ) {
        println!("cargo:rustc-cfg=unstable_channel");
    }

    Ok(())
}
