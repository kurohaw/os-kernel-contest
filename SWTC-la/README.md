# SWTC LoongArch Kernel

This directory contains the LoongArch evaluation kernel for SWTC.

It is adapted from the open-source StarryX project and its ArceOS base:

- StarryX: https://github.com/Anekoique/StarryX
- ArceOS: https://github.com/arceos-org/arceos
- Imported StarryX baseline: commit `d77359efece4f3216dc2cfac5165b68d1d679923`

The SWTC adaptation adds an offline build, compatibility with
`nightly-2025-05-20`, a corrected LoongArch early boot stack address, official
EXT4 test-image startup, SWTC branding, deterministic shutdown, and relative
`execve` path handling.

After editing vendored source on Windows, rebuild release checksums with
`python tools/vendor_checksums.py --fix --vendor SWTC-la/vendor --source index`
from the repository root.

The original authors and licenses are preserved. The imported code remains
available under the upstream GPL-3.0-or-later, Apache-2.0, or MulanPSL-2.0
terms. The bundled lwext4 binding and C implementation retain their own
GPL-2.0 license files.

Local validation with the official `pre-20250615` LoongArch image completed
all 32 musl basic tests and all 32 glibc basic tests with matching START/END
markers. This is local evidence, not an official online score.

The root build can use Cargo `-Z build-std` when the LoongArch target libraries
are absent but `rust-src` is installed. The default build still falls back to
the RISC-V placeholder if the real LoongArch build fails, so RISC-V evaluation
remains available.

After both basic groups, the embedded init script probes and runs the official
BusyBox, Lua, glibc/musl libcbench, musl libctest, and a bounded LTP subset.
Libcbench has a 180-second group timeout, libctest has a 300-second group
timeout, and each LTP case has a 5-second timeout. A local QEMU 9.2.1 run with
the official `pre-2025` static musl BusyBox completed all 55 commands with no
failure, panic, or unsupported syscall and shut down in about 135 seconds.
Lua, libcbench, libctest, LTP, and all online LoongArch scores still require
official evaluator confirmation.
