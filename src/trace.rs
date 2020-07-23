use crate::syscall::Syscall;

use std::thread;

pub struct Trace {
    pid: i32,
    incall: bool,
}

pub fn spawn_trace(pid: i32) {
    thread::spawn(move || {
        let mut trace = Trace::new(pid as i32);
        trace.trace();
    });
}

impl Trace {
    pub fn new(pid: i32) -> Trace {
        Trace { pid, incall: false }
    }
    pub fn pid(&self) -> i32 {
        self.pid
    }
    pub fn trace(&mut self) {
        let data = libc::PTRACE_O_TRACEEXIT
            | libc::PTRACE_O_TRACEEXEC
            | libc::PTRACE_O_EXITKILL
            | libc::PTRACE_O_TRACEFORK
            | libc::PTRACE_O_TRACEVFORK
            | libc::PTRACE_O_TRACECLONE;
        unsafe {
            // println!("ptrace {}", self.pid());
            let ret = libc::ptrace(libc::PTRACE_ATTACH, self.pid(), 0, data);
            // println!("ret = {}", ret);
        }
        loop {
            let regs = libc::user_regs_struct {
                r15: 0,
                r14: 0,
                r13: 0,
                r12: 0,
                rbp: 0,
                rbx: 0,
                r11: 0,
                r10: 0,
                r9: 0,
                r8: 0,
                rax: 0,
                rcx: 0,
                rdx: 0,
                rsi: 0,
                rdi: 0,
                orig_rax: 0,
                rip: 0,
                cs: 0,
                eflags: 0,
                rsp: 0,
                ss: 0,
                fs_base: 0,
                gs_base: 0,
                ds: 0,
                es: 0,
                fs: 0,
                gs: 0,
            };
            let mut status: i32 = 0;
            unsafe {
                libc::waitpid(self.pid, &mut status, libc::WSTOPPED);
                if libc::WIFEXITED(status) {
                    // println!("{} exited!", self.pid);
                    break;
                }
                libc::ptrace(libc::PTRACE_GETREGS, self.pid, 0, &regs);
                // println!("syscall = {}, status = {}", regs.orig_rax, status);
            }
            self.incall = -libc::ENOSYS == regs.rax as i32;
            if self.incall {
                // 进入系统调用
                if regs.orig_rax == Syscall::SYS_ptrace as u64 {
                    println!("{} use ptrace syscall", self.pid);
                }
                // if regs.orig_rax == 120 {
                //     regs.orig_rax = 399;
                //     regs.rax = libc::EPERM as u64;
                //     libc::ptrace(libc::PTRACE_SETREGS, pid, 0, &regs);
                // }
                // println!("{}", regs.orig_rax);
            } else {
                if regs.orig_rax == Syscall::SYS_clone as u64
                    || regs.orig_rax == Syscall::SYS_fork as u64
                    || regs.orig_rax == Syscall::SYS_vfork as u64
                {
                    let pid = self.return_code();
                    // 监控子进程
                    spawn_trace(pid as i32);
                }
                else if regs.orig_rax == Syscall::SYS_ptrace as u64 {
                    unsafe {
                        libc::ptrace(libc::PTRACE_POKEUSER, self.pid, regs.rax * 8, 0);
                    }
                }
            }
            unsafe {
                // 将系统调用传递给调试的进程
                libc::ptrace(libc::PTRACE_SYSCALL, self.pid, 0, 0);
            }
        }
    }
    pub fn return_code(&self) -> i64 {
        unsafe { return libc::ptrace(libc::PTRACE_PEEKUSER, self.pid, 8 * libc::RAX, 0) }
    }
}
