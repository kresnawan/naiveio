use naiveio::{spawn, Runtime};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[test]
fn spawned_task_runs_to_completion() {
    let mut rt = Runtime::new("rt1");
    let _guard = rt.enter();

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();

    spawn(async move {
        done_clone.store(true, Ordering::SeqCst);
    });

    rt.run();

    assert!(done.load(Ordering::SeqCst), "task must be finished after run()");
}

#[test]
fn run_stops_when_all_tasks_finish() {
    let mut rt = Runtime::new("rt1");
    let _guard = rt.enter();

    spawn(async {});
    spawn(async {});

    rt.run();
}

#[test]
#[should_panic(expected = "Runtime not found")]
fn spawn_without_enter_panics() {
    spawn(async {});
}