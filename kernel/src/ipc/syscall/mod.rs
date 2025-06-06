use alloc::sync::Arc;
use system_error::SystemError;

use crate::{
    arch::ipc::signal::{SigSet, Signal},
    ipc::signal_types::SigInfo,
    process::ProcessControlBlock,
    syscall::Syscall,
    time::{
        timekeep::{ktime_t, KTIME_MAX, KTIME_SEC_MAX},
        PosixTimeSpec, NSEC_PER_SEC,
    },
};

pub mod sys_kill;
pub mod sys_pipe2;
mod sys_restart;
mod sys_rt_sigprocmask;
mod sys_shmat;
mod sys_shmctl;
mod sys_shmdt;
mod sys_shmget;
mod sys_sigaction;
mod sys_sigpending;

#[cfg(target_arch = "x86_64")]
pub mod sys_pipe;

impl Syscall {
    pub fn do_sigtimedwait(
        pcb: Arc<ProcessControlBlock>,
        which: SigSet,
        timespec: Option<PosixTimeSpec>,
        info: &mut Option<SigInfo>,
    ) -> Result<i32, SystemError> {
        let mut expires: ktime_t = 0;
        if let Some(ts) = timespec {
            if ts.tv_sec < 0 || ts.tv_nsec > NSEC_PER_SEC as i64 {
                return Err(SystemError::EINVAL);
            }
            if ts.tv_sec >= KTIME_SEC_MAX {
                expires = KTIME_MAX;
            } else {
                expires = ts.tv_sec * NSEC_PER_SEC as i64 + ts.tv_nsec;
            }
        }

        let mut mask = which;
        mask.remove(Signal::SIGKILL.into_sigset() | Signal::SIGSTOP.into_sigset());
        let _ = mask.complement();

        let mut sig_num: Option<Signal> = None;

        {
            let mut sig_info_guard = pcb.sig_info_mut();
            let (sig, opt_info) = sig_info_guard.dequeue_signal(&mask, &pcb);
            if sig != Signal::INVALID {
                sig_num = Some(sig);
                *info = opt_info;
            }
        }

        if let Some(sig) = sig_num {
            return Ok(sig as i32);
        }

        if expires == 0 {
            return Err(SystemError::EAGAIN_OR_EWOULDBLOCK);
        }

        // 如果没有信号并且有超时时间，设置pcb_sig_flag,进入可中断睡眠状态，等待信号或超时,唤醒后恢复pcb_sig_flag，再次尝试获取信号
        todo!();
    }
}
