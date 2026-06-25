
/// The main timer

use crate::qa7_control::{CoreInterruptSource, QA7Control};
//use aarch64::{asm, regs::*};
use aarch64_cpu::{asm, registers::*};
use tock_registers::interfaces::Writeable;
use tock_registers::interfaces::Readable;
use core::arch::asm;

/// The Raspberry Pi timer.
pub trait BasicTimer {
    /// The timer frequency (Hz)
    fn freq() -> u64;

    /// Returns a new instance.
    fn new() -> Self;

    /// Initialization timer.
    fn init(&mut self);

    /// Stop timer.
    fn stop(&mut self);

    /// Reads the timer's counter and returns the 64-bit counter value.
    /// The returned value is the number of elapsed microseconds.
    fn read(&self) -> u64;

    /// Sets up a match in timer 1 to occur `us` microseconds from now. If
    /// interrupts for timer 1 are enabled and IRQs are unmasked, then a timer
    /// interrupt will be issued in `us` microseconds.
    fn tick_in(&mut self, us: usize);

    /// Returns `true` if timer interruption is pending. Otherwise, returns `false`.
    fn is_pending(&self) -> bool;
}

/// wait for `cycle` CPU cycles
#[inline(always)]
pub fn delay(cycle: usize) {
    for _ in 0..cycle {
        unsafe { asm!("nop") }
    }
}

/// Core timers interrupts (ref: QA7 4.6, page 13)
#[repr(u8)]
#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Copy, Clone, PartialEq, Debug)]

enum CoreTimerControl {
    CNTPSIRQ = 0,
    CNTPNSIRQ = 1,
    CNTHPIRQ = 2,
    CNTVIRQ = 3,
}

/// The ARM generic timer.
pub struct GenericTimer {
    control: QA7Control,
}

impl BasicTimer for GenericTimer {
    #[inline]
    fn freq() -> u64 {
        // 62500000 on qemu, 19200000 on real machine
        CNTFRQ_EL0.get() as u64
    }

    #[inline]
    fn new() -> Self {
        GenericTimer {
            control: QA7Control::new(),
        }
    }

    #[inline]
    fn init(&mut self) {
        self.control.registers.CORE_TIMER_IRQCNTL[cpuid()]
            .write(1 << (CoreTimerControl::CNTPNSIRQ as u8));
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    }

    #[inline]
    fn stop(&mut self) {
        self.control.registers.CORE_TIMER_IRQCNTL[cpuid()].write(0);
    }

    #[inline]
    fn read(&self) -> u64 {
        (CNTPCT_EL0.get() * 1000000 / Self::freq()) as u64
    }

    #[inline]
    fn tick_in(&mut self, us: usize) {
        let count = Self::freq() * (us as u64) / 1000000;
        // max `68719476` us (0xffff_ffff / 38400000 * 62500000).
        debug_assert!(count <= u32::max_value() as u64);
        CNTP_TVAL_EL0.set(count);
    }

    #[inline]
    fn is_pending(&self) -> bool {
        self.control
            .is_irq_pending(cpuid(), CoreInterruptSource::CNTPNSIRQ)
    }
}

pub type Timer = GenericTimer;

/// wait for us microsecond
pub fn delay_us(us: usize) {
    let mut timer = Timer::new();
    timer.init();
    let start_time = timer.read();
    while timer.read() - start_time < us as u64 {
        unsafe { asm!("nop") }
    }
}

#[inline]
pub fn cpuid() -> usize {
    (MPIDR_EL1.get() & 3) as usize
}
