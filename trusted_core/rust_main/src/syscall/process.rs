use crate::kprintln;
use crate::scheduler;

pub(super) fn sys_exit(exit_code: i32) {
    kprintln!("exit with code {}", exit_code);
    scheduler::batch::mark_current_exited();
    scheduler::batch::load_next_and_run();
}
