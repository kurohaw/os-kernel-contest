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

cd /musl
/bin/sh ./basic_testcode.sh

cd /glibc
/bin/sh ./basic_testcode.sh

/bin/sync
