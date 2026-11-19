// Copyright The async-profiler authors
// SPDX-License-Identifier: Apache-2.0
//
// Standalone Linux x86-64 workload for LD_PRELOAD profiling.
// Compile with the latest nightly:
// rustc +nightly --edition=2021 -C opt-level=2 -C debuginfo=2 -C force-unwind-tables=yes \
//   rust_preload_probe.rs
// Load the profiler solely through LD_PRELOAD and ASPROF_COMMAND.
#![feature(rustc_private)]
extern crate libc;

use std::{hint::black_box, mem, ptr, thread, time::{Duration, Instant}};

#[inline(never)]
fn busy_work(mut value: u64) -> u64 {
    for _ in 0..65_536 {
        value = black_box(value.wrapping_mul(6364136223846793005).wrapping_add(1));
    }
    value
}

fn main() {
    let seconds: u64 = std::env::args().nth(1).unwrap_or_else(|| "610".into())
        .parse().expect("duration in seconds");
    assert!((1..=660).contains(&seconds), "duration must be 1..=660 seconds");
    thread::spawn(move || {
        unsafe {
            let mut stack: libc::stack_t = mem::zeroed();
            let mut mask: libc::sigset_t = mem::zeroed();
            assert_eq!(libc::sigaltstack(ptr::null(), &mut stack), 0);
            assert_eq!(libc::pthread_sigmask(libc::SIG_BLOCK, ptr::null(), &mut mask), 0);
            let min_sigstack_size = libc::getauxval(libc::AT_MINSIGSTKSZ);
            println!("pid={} tid={} ss_sp={:p} ss_size={} AT_MINSIGSTKSZ={min_sigstack_size} ss_flags={}",
                     std::process::id(), libc::syscall(libc::SYS_gettid),
                     stack.ss_sp, stack.ss_size, stack.ss_flags);
            for (name, signal, peer) in [
                ("CPU", libc::SIGPROF, libc::SIGVTALRM),
                ("wall", libc::SIGVTALRM, libc::SIGPROF),
            ] {
                assert_eq!(libc::sigismember(&mask, signal), 0, "sampling signal blocked");
                let mut action: libc::sigaction = mem::zeroed();
                assert_eq!(libc::sigaction(signal, ptr::null(), &mut action), 0);
                println!("{name}: SA_ONSTACK={} masks_peer={}",
                         action.sa_flags & libc::SA_ONSTACK != 0,
                         libc::sigismember(&action.sa_mask, peer) == 1);
            }
        }
        println!("Running workload for {seconds}s");
        let deadline = Instant::now() + Duration::from_secs(seconds);
        let mut value = 1;
        while Instant::now() < deadline {
            value = busy_work(value);
        }
        println!("Completed: checksum={value}");
    }).join().unwrap();
}
