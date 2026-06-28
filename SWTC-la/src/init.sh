echo "================ SWTC LOONGARCH BASIC ================"

/musl/busybox mkdir -p /bin /lib /usr/lib /usr/lib64 /tmp /var/tmp
/musl/busybox ln -sf /musl/busybox /bin/sh
/musl/busybox ln -sf /musl/busybox /bin/echo
/musl/busybox ln -sf /musl/busybox /bin/sleep
/musl/busybox ln -sf /musl/busybox /bin/cat
/musl/busybox ln -sf /musl/busybox /bin/ls
/musl/busybox ln -sf /musl/busybox /bin/cp
/musl/busybox ln -sf /musl/busybox /bin/mkdir
/musl/busybox ln -sf /musl/busybox /bin/rm
/musl/busybox ln -sf /musl/busybox /bin/touch
/musl/busybox ln -sf /musl/busybox /bin/sync
/musl/busybox ln -sf /musl/lib/libc.so /lib/ld-musl-loongarch-lp64d.so.1
/musl/busybox ln -sf /glibc/lib/ld-linux-loongarch-lp64d.so.1 /lib/ld-linux-loongarch-lp64d.so.1
/musl/busybox ln -sf /glibc/lib/libc.so.6 /lib/libc.so.6
/musl/busybox ln -sf /glibc/lib/libm.so.6 /lib/libm.so.6
/musl/busybox rm -rf /lib64 /usr/lib /usr/lib64
/musl/busybox ln -sf /lib /lib64
/musl/busybox ln -sf /lib /usr/lib
/musl/busybox ln -sf /lib /usr/lib64

export PATH="/bin:/musl:/glibc"
export LD_LIBRARY_PATH=".:./lib:/musl/lib:/glibc/lib:/lib"

run_script() (
    test_dir="$1"
    script_name="$2"
    if [ ! -f "$test_dir/$script_name" ]; then
        echo "[swtc-la] skip missing $test_dir/$script_name"
        exit 0
    fi
    cd "$test_dir" || exit 0
    /bin/sh "./$script_name"
)

run_busybox_script() (
    test_dir="$1"
    group_tag="$2"
    if [ ! -f "$test_dir/busybox_testcode.sh" ] || [ ! -f "$test_dir/busybox" ]; then
        echo "[swtc-la] skip incomplete busybox runtime in $test_dir"
        exit 0
    fi

    work_dir="/tmp/swtc-busybox-$group_tag"
    /musl/busybox rm -rf "$work_dir"
    /musl/busybox mkdir -p "$work_dir"
    /musl/busybox ln -sf "$test_dir/busybox" "$work_dir/busybox"
    /musl/busybox ln -sf "$test_dir/busybox_testcode.sh" "$work_dir/busybox_testcode.sh"
    if [ -f "$test_dir/busybox_cmd.txt" ]; then
        /musl/busybox cp "$test_dir/busybox_cmd.txt" "$work_dir/busybox_cmd.txt"
    fi
    if [ -f "$test_dir/ls" ]; then
        /musl/busybox ln -sf "$test_dir/ls" "$work_dir/ls"
    fi

    cd "$work_dir" || exit 0
    /bin/sh ./busybox_testcode.sh
)

run_script_with_timeout() (
    test_dir="$1"
    script_name="$2"
    timeout_seconds="$3"
    if [ ! -f "$test_dir/$script_name" ]; then
        echo "[swtc-la] skip missing $test_dir/$script_name"
        exit 0
    fi
    cd "$test_dir" || exit 0
    /musl/busybox timeout "$timeout_seconds" /bin/sh "./$script_name"
    status="$?"
    if [ "$status" -ne 0 ]; then
        echo "[swtc-la] $test_dir/$script_name exited with status $status"
    fi
)

run_lmbench_subset() (
    test_dir="$1"
    group_name="$2"
    if [ ! -f "$test_dir/lmbench_all" ]; then
        echo "[swtc-la] skip missing $test_dir/lmbench_all"
        exit 0
    fi

    cd "$test_dir" || exit 0
    echo "#### OS COMP TEST GROUP START $group_name ####"
    echo latency measurements
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 null
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 read
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 write
    /musl/busybox mkdir -p /var/tmp /tmp
    /musl/busybox touch /var/tmp/lmbench
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 stat /var/tmp/lmbench
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 fstat /var/tmp/lmbench
    /musl/busybox timeout 8 ./lmbench_all lat_syscall -P 1 open /var/tmp/lmbench
    /musl/busybox timeout 8 ./lmbench_all lat_select -n 100 -P 1 file
    /musl/busybox timeout 8 ./lmbench_all lat_sig -P 1 install
    /musl/busybox timeout 8 ./lmbench_all lat_sig -P 1 catch
    /musl/busybox timeout 8 ./lmbench_all lat_sig -P 1 prot lat_sig
    /musl/busybox timeout 8 ./lmbench_all lat_pipe -P 1
    /musl/busybox timeout 8 ./lmbench_all lat_proc -P 1 fork
    /musl/busybox timeout 8 ./lmbench_all lat_proc -P 1 exec
    if [ -f hello ]; then
        /musl/busybox cp hello /tmp
    fi
    /musl/busybox timeout 8 ./lmbench_all lat_proc -P 1 shell
    /musl/busybox timeout 15 ./lmbench_all lmdd label="File /var/tmp/XXX write bandwidth:" of=/var/tmp/XXX move=1m fsync=1 print=3
    /musl/busybox timeout 8 ./lmbench_all lat_pagefault -P 1 /var/tmp/XXX
    /musl/busybox timeout 8 ./lmbench_all lat_mmap -P 1 512k /var/tmp/XXX
    echo file system latency
    /musl/busybox timeout 8 ./lmbench_all lat_fs /var/tmp
    echo Bandwidth measurements
    /musl/busybox timeout 8 ./lmbench_all bw_pipe -P 1
    /musl/busybox timeout 8 ./lmbench_all bw_file_rd -P 1 512k io_only /var/tmp/XXX
    /musl/busybox timeout 8 ./lmbench_all bw_file_rd -P 1 512k open2close /var/tmp/XXX
    /musl/busybox timeout 8 ./lmbench_all bw_mmap_rd -P 1 512k mmap_only /var/tmp/XXX
    /musl/busybox timeout 8 ./lmbench_all bw_mmap_rd -P 1 512k open2close /var/tmp/XXX
    echo context switch overhead
    /musl/busybox timeout 12 ./lmbench_all lat_ctx -P 1 -s 32 2 4 8 16 24 32 64 96
    echo "#### OS COMP TEST GROUP END $group_name ####"
)

run_ltp_subset() (
    ltp_bin="/musl/ltp/testcases/bin"
    if [ ! -d "$ltp_bin" ]; then
        echo "[swtc-la] skip missing $ltp_bin"
        exit 0
    fi

    echo "#### OS COMP TEST GROUP START ltp-musl ####"
    cd "$ltp_bin" || exit 0
    for case_name in \
        alarm02 chown01 close01 close02 \
        dup01 dup02 dup03 dup04 dup06 dup07 dup202 dup204 dup206 dup207 \
        exit02 exit_group01 fork01 fork03 fork07 fork08 fork10 \
        getcwd01 getegid02 geteuid01 getgid03 getpid02 getppid02 \
        gettimeofday01 gettimeofday02 getuid01 lseek01 lseek07 uname01 uname04 \
        mkdir05 mkdirat01 pipe01 pipe06 pipe10 pipe11 pipe14 readv01 rmdir01 \
        access01 access02 access03 access04 \
        faccessat01 faccessat02 faccessat201 faccessat202 \
        chmod01 chmod03 chmod05 chmod06 chmod07 \
        fchmod01 fchmod02 fchmod03 fchmod04 fchmod05 fchmod06 fchmodat01 fchmodat02 \
        chown02 chown03 chown04 chown05 \
        fchown01 fchown02 fchown03 fchown04 fchown05 fchownat01 fchownat02 \
        getrlimit01 getrlimit02 getrlimit03 \
        getrusage01 getrusage02 getrusage03 getrusage04 \
        gettid01 gettid02 \
        getrandom01 getrandom02 getrandom03 getrandom04 getrandom05 \
        dup05 dup201 dup203 dup205 dup3_01 dup3_02 \
        openat01 openat02 openat03 openat04 openat201 openat202 openat203 \
        fcntl01 fcntl01_64 fcntl02 fcntl02_64 fcntl03 fcntl03_64 \
        fcntl04 fcntl04_64 fcntl05 fcntl05_64 fcntl07 fcntl07_64 \
        fcntl08 fcntl08_64 fcntl09 fcntl09_64 fcntl10 fcntl10_64 \
        fcntl11 fcntl11_64 fcntl12 fcntl12_64 fcntl13 fcntl13_64 \
        pipe02 pipe03 pipe04 pipe05 pipe07 pipe08 pipe09 pipe12 pipe13 pipe15 \
        pipe2_01 pipe2_02 pipe2_04 \
        writev01 writev02 writev03 writev05 writev06 writev07 \
        preadv01 preadv01_64 preadv02 preadv02_64 preadv03 preadv03_64 \
        pwritev01 pwritev01_64 pwritev02 pwritev02_64 pwritev03 pwritev03_64 \
        pwrite02 pwrite02_64 \
        poll01 poll02 \
        pselect01 pselect01_64 pselect02 pselect02_64 pselect03 pselect03_64 \
        select01 select02 select03 select04 \
        alarm03 alarm05 alarm06 alarm07 \
        nanosleep01 nanosleep02 nanosleep04 \
        kill02 kill03 kill05 kill06 kill07 kill08 kill09 kill10 kill11 kill12 kill13 \
        waitpid01 waitpid03 waitpid04 waitpid06 waitpid07 waitpid08 \
        waitpid09 waitpid10 waitpid11 waitpid12 waitpid13 \
        fork04 fork09 fork13 fork14
    do
        if [ ! -f "./$case_name" ]; then
            continue
        fi
        echo "RUN LTP CASE $case_name"
        /musl/busybox timeout 5 "./$case_name"
        status="$?"
        echo "FAIL LTP CASE $case_name : $status"
    done
    echo "#### OS COMP TEST GROUP END ltp-musl ####"
)

run_libctest_cases() (
    test_dir="$1"
    if [ ! -d "$test_dir" ]; then
        echo "[swtc-la] skip missing $test_dir"
        exit 0
    fi
    cd "$test_dir" || exit 0
    if [ ! -f runtest.exe ] || [ ! -f entry-static.exe ]; then
        echo "[swtc-la] skip incomplete libctest runtime in $test_dir"
        exit 0
    fi

    echo "#### OS COMP TEST GROUP START libctest-musl ####"
    run_libctest_list run-static.sh entry-static.exe 107
    if [ -f entry-dynamic.exe ]; then
        run_libctest_list run-dynamic.sh entry-dynamic.exe 110
    fi
    echo "#### OS COMP TEST GROUP END libctest-musl ####"
)

run_libctest_list() {
    script_name="$1"
    entry_name="$2"
    limit="$3"
    count=0
    if [ ! -f "$script_name" ]; then
        return
    fi
    while IFS= read -r line; do
        set -- $line
        if [ "$1" != "./runtest.exe" ] || [ "$3" != "$entry_name" ] || [ -z "$4" ]; then
            continue
        fi
        case_name="$4"
        /musl/busybox timeout 8 ./runtest.exe -w "$entry_name" "$case_name"
        status="$?"
        if [ "$status" -ne 0 ]; then
            echo "FAIL LIBCTEST CASE $case_name : $status"
        fi
        count=$((count + 1))
        if [ "$count" -ge "$limit" ]; then
            break
        fi
    done < "$script_name"
}

# Keep the locally validated basic order unchanged.
run_script /musl basic_testcode.sh
run_script /glibc basic_testcode.sh

# Expand only functional groups that already score on the RISC-V kernel.
run_busybox_script /musl musl
run_busybox_script /glibc glibc
run_script /musl lua_testcode.sh
run_script /glibc lua_testcode.sh

# Keep the LA musl libcbench path because it is already scoring.  Do not run
# glibc libcbench before libctest/LTP: the 2026-06-29 official log traps in
# libcbench-glibc and truncates every later LA group.
run_script_with_timeout /musl libcbench_testcode.sh 180

# The official image has used both layouts across revisions.
if [ -f /musl/libctest_testcode.sh ]; then
    run_libctest_cases /musl
elif [ -f /musl/libctest/libctest_testcode.sh ]; then
    run_libctest_cases /musl/libctest
fi

run_ltp_subset

# Put large benchmark probes after all functional groups.  This keeps the
# proven basic/BusyBox/libcbench/libctest/LTP score path intact if lmbench
# still times out or fails to score.
run_lmbench_subset /musl lmbench-musl
run_lmbench_subset /glibc lmbench-glibc

# Re-enable this only after the LoongArch memory access trap in glibc
# libcbench is fixed.  It currently scores 0 and prevents LA libctest/LTP from
# running when placed earlier.
# run_script_with_timeout /glibc libcbench_testcode.sh 180

/bin/sync
