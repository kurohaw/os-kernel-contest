echo "================ SWTC LOONGARCH BASIC ================"

/musl/busybox mkdir -p /bin /lib /usr/lib /usr/lib64 /var/tmp
/musl/busybox ln -sf /musl/busybox /bin/sh
/musl/busybox ln -sf /musl/busybox /bin/echo
/musl/busybox ln -sf /musl/busybox /bin/sleep
/musl/busybox ln -sf /musl/busybox /bin/cat
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
        gettimeofday01 getuid01 lseek01 lseek07 uname01 uname04
    do
        if [ ! -f "./$case_name" ]; then
            continue
        fi
        echo "RUN LTP CASE $case_name"
        /musl/busybox timeout 5 "./$case_name"
        status="$?"
        if [ "$status" -ne 0 ]; then
            echo "FAIL LTP CASE $case_name : $status"
        fi
    done
    echo "#### OS COMP TEST GROUP END ltp-musl ####"
)

# Keep the locally validated basic order unchanged.
run_script /musl basic_testcode.sh
run_script /glibc basic_testcode.sh

# Expand only functional groups that already score on the RISC-V kernel.
run_script /musl busybox_testcode.sh
run_script /glibc busybox_testcode.sh
run_script /musl lua_testcode.sh
run_script /glibc lua_testcode.sh

# The official image has used both layouts across revisions.
if [ -f /musl/libctest_testcode.sh ]; then
    run_script_with_timeout /musl libctest_testcode.sh 300
elif [ -f /musl/libctest/libctest_testcode.sh ]; then
    run_script_with_timeout /musl/libctest libctest_testcode.sh 300
fi

run_ltp_subset

/bin/sync
