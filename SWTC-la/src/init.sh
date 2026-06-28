echo "================ SWTC LOONGARCH BASIC ================"

/musl/busybox mkdir -p /bin /lib /usr/lib /usr/lib64 /tmp /var/tmp
/musl/busybox ln -sf /musl/busybox /bin/sh
/musl/busybox ln -sf /musl/busybox /bin/echo
/musl/busybox ln -sf /musl/busybox /bin/sleep
/musl/busybox ln -sf /musl/busybox /bin/cat
/musl/busybox ln -sf /musl/busybox /bin/ls
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
        mkdir05 mkdirat01 pipe01 pipe06 pipe10 pipe11 pipe14 readv01 rmdir01
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
        /musl/busybox timeout 3 ./runtest.exe -w "$entry_name" "$case_name"
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
run_script /musl busybox_testcode.sh
run_script /glibc busybox_testcode.sh
run_script /musl lua_testcode.sh
run_script /glibc lua_testcode.sh

# Keep performance tests isolated in their own official directories.  The
# libcbench scripts are already part of the stable RISC-V score baseline and
# require no global runtime files beyond the links prepared above.
run_script_with_timeout /musl libcbench_testcode.sh 180
run_script_with_timeout /glibc libcbench_testcode.sh 180

# The official image has used both layouts across revisions.
if [ -f /musl/libctest_testcode.sh ]; then
    run_libctest_cases /musl
elif [ -f /musl/libctest/libctest_testcode.sh ]; then
    run_libctest_cases /musl/libctest
fi

run_ltp_subset

/bin/sync
