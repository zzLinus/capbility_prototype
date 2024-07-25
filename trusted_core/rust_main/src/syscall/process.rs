use crate::scheduler;
use log::info;

pub(super) fn sys_exit(exit_code: i32) {
    info!("exit with code {}", exit_code);
    scheduler::batch::mark_current_exited();
    scheduler::batch::load_next_and_run();
}