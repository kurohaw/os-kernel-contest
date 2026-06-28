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

run_iozone_subset() (
    test_dir="$1"
    group_name="$2"
    if [ ! -f "$test_dir/iozone" ]; then
        echo "[swtc-la] skip missing $test_dir/iozone"
        exit 0
    fi
    cd "$test_dir" || exit 0
    echo "#### OS COMP TEST GROUP START $group_name ####"
    /musl/busybox echo iozone automatic measurements
    /musl/busybox timeout 30 ./iozone -a -r 1k -s 4m
    /musl/busybox echo iozone throughput write/read measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 0 -i 1 -r 1k -s 1m
    /musl/busybox echo iozone throughput random-read measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 0 -i 2 -r 1k -s 1m
    /musl/busybox echo iozone throughput read-backwards measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 0 -i 3 -r 1k -s 1m
    /musl/busybox echo iozone throughput stride-read measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 0 -i 5 -r 1k -s 1m
    /musl/busybox echo iozone throughput fwrite/fread measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 6 -i 7 -r 1k -s 1m
    /musl/busybox echo iozone throughput pwrite/pread measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 9 -i 10 -r 1k -s 1m
    /musl/busybox echo iozone throughtput pwritev/preadv measurements
    /musl/busybox timeout 25 ./iozone -t 2 -i 11 -i 12 -r 1k -s 1m
    echo "#### OS COMP TEST GROUP END $group_name ####"
)

run_iperf_subset() (
    test_dir="$1"
    group_name="$2"
    port="$3"
    if [ ! -f "$test_dir/iperf3" ]; then
        echo "[swtc-la] skip missing $test_dir/iperf3"
        exit 0
    fi
    cd "$test_dir" || exit 0
    echo "#### OS COMP TEST GROUP START $group_name ####"
    ./iperf3 -s -p "$port" > /tmp/iperf-server.log 2>&1 &
    server_pid=$!
    /musl/busybox sleep 1
    run_iperf_case BASIC_UDP "-u -b 1000G" "$port"
    run_iperf_case BASIC_TCP "" "$port"
    run_iperf_case PARALLEL_UDP "-u -P 2 -b 1000G" "$port"
    run_iperf_case PARALLEL_TCP "-P 2" "$port"
    run_iperf_case REVERSE_UDP "-u -R -b 1000G" "$port"
    run_iperf_case REVERSE_TCP "-R" "$port"
    kill -9 "$server_pid" 2>/dev/null
    echo "#### OS COMP TEST GROUP END $group_name ####"
)

run_iperf_case() {
    name="$1"
    args="$2"
    port="$3"
    echo "====== iperf $name begin ======"
    /musl/busybox timeout 8 ./iperf3 -c 127.0.0.1 -p "$port" -t 2 -i 0 $args
    if [ "$?" = 0 ]; then
        ans="success"
    else
        ans="fail"
    fi
    echo "====== iperf $name end: $ans ======"
    echo ""
}

run_netperf_subset() (
    test_dir="$1"
    group_name="$2"
    port="$3"
    if [ ! -f "$test_dir/netserver" ] || [ ! -f "$test_dir/netperf" ]; then
        echo "[swtc-la] skip incomplete netperf runtime in $test_dir"
        exit 0
    fi
    cd "$test_dir" || exit 0
    echo "#### OS COMP TEST GROUP START $group_name ####"
    ./netserver -D -L 127.0.0.1 -p "$port" &
    server_pid=$!
    /musl/busybox sleep 1
    run_netperf_case UDP_STREAM "-s 16k -S 16k -m 1k -M 1k" "$port"
    run_netperf_case TCP_STREAM "-s 16k -S 16k -m 1k -M 1k" "$port"
    run_netperf_case UDP_RR "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1" "$port"
    run_netperf_case TCP_RR "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1" "$port"
    run_netperf_case TCP_CRR "-s 16k -S 16k -m 1k -M 1k -r 64,64 -R 1" "$port"
    kill -9 "$server_pid" 2>/dev/null
    echo "#### OS COMP TEST GROUP END $group_name ####"
)

run_netperf_case() {
    name="$1"
    args="$2"
    port="$3"
    echo "====== netperf $name begin ======"
    /musl/busybox timeout 8 ./netperf -H 127.0.0.1 -p "$port" -t "$name" -l 1 -- $args
    if [ "$?" = 0 ]; then
        ans="success"
    else
        ans="fail"
    fi
    echo "====== netperf $name end: $ans ======"
}

run_cyclictest_nostress() (
    test_dir="$1"
    group_name="$2"
    if [ ! -f "$test_dir/cyclictest" ]; then
        echo "[swtc-la] skip missing $test_dir/cyclictest"
        exit 0
    fi
    cd "$test_dir" || exit 0
    echo "#### OS COMP TEST GROUP START $group_name ####"
    run_cyclictest_case NO_STRESS_P1 "-a -i 1000 -t1 -p99 -D 1s -q"
    run_cyclictest_case NO_STRESS_P8 "-a -i 1000 -t8 -p99 -D 1s -q"
    echo "#### OS COMP TEST GROUP END $group_name ####"
)

run_cyclictest_case() {
    name="$1"
    args="$2"
    echo "====== cyclictest $name begin ======"
    /musl/busybox timeout 8 ./cyclictest $args
    if [ "$?" = 0 ]; then
        ans="success"
    else
        ans="fail"
    fi
    echo "====== cyclictest $name end: $ans ======"
}

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
        dup203 fcntl05 fcntl13 flock01 flock02 flock03 \
        exit02 exit_group01 fork01 fork03 fork07 fork08 fork10 \
        getcwd01 getegid02 geteuid01 getgid03 getpid02 getppid02 \
        gettimeofday01 gettimeofday02 getuid01 lseek01 lseek07 uname01 uname04 \
        getitimer02 getrandom02 kill02 kill03 kill09 \
        mkdir05 mkdirat01 nanosleep01 openat201 pipe01 pipe03 pipe06 pipe09 pipe10 pipe11 pipe12 pipe14 pipe15 \
        preadv01 preadv01_64 pwrite02 pwrite02_64 readv01 rmdir01 rmdir03 \
        statx02 statx12 timerfd02 truncate02 utime04 waitpid03
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

# Official queues are slow to evaluate online, so batch the large zero-score
# groups behind the stable functional groups.  Each item is individually
# time-limited so one broken benchmark should not block shutdown.
run_lmbench_subset /musl lmbench-musl
run_lmbench_subset /glibc lmbench-glibc
run_iozone_subset /musl iozone-musl
run_iozone_subset /glibc iozone-glibc
run_iperf_subset /musl iperf-musl 5001
run_iperf_subset /glibc iperf-glibc 5002
run_netperf_subset /musl netperf-musl 12865
run_netperf_subset /glibc netperf-glibc 12866
run_cyclictest_nostress /musl cyclictest-musl
run_cyclictest_nostress /glibc cyclictest-glibc

/bin/sync
