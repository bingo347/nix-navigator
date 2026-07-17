use parking_lot::{Condvar, Mutex, Once};
use std::{
    alloc::{self, Layout},
    collections::VecDeque,
    hint,
    num::NonZeroUsize,
    panic::{self, UnwindSafe},
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
    thread,
};

#[repr(transparent)]
pub struct JoinHandle<T> {
    result_ptr: Arc<AtomicPtr<thread::Result<T>>>,
}

static INIT: Once = Once::new();
static QUEUE: Mutex<VecDeque<Box<dyn FnOnce() + Send>>> = Mutex::new(VecDeque::new());
static CV: Condvar = Condvar::new();

pub fn spawn<T: Send + 'static>(
    task: impl FnOnce() -> T + Send + UnwindSafe + 'static,
) -> JoinHandle<T> {
    init();
    let result_ptr = Arc::new(AtomicPtr::<thread::Result<T>>::default());
    let task_wrap = Box::new({
        let result_ptr = result_ptr.clone();
        move || {
            let result = panic::catch_unwind(task);
            result_ptr.store(Box::into_raw(Box::new(result)), Ordering::Release);
        }
    });
    QUEUE.lock().push_back(task_wrap);
    CV.notify_one();

    JoinHandle { result_ptr }
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> T {
        let result_ptr = self.result_ptr;
        loop {
            let result_ptr = result_ptr.load(Ordering::Acquire);
            if !result_ptr.is_null() {
                let result = unsafe { result_ptr.read() };
                unsafe {
                    alloc::dealloc(result_ptr.cast(), Layout::new::<thread::Result<T>>());
                }
                match result {
                    Ok(result) => return result,
                    Err(err) => panic::resume_unwind(err),
                }
            }
            hint::spin_loop();
        }
    }
}

fn init() {
    INIT.call_once(|| {
        let threads_count = thread::available_parallelism().map_or(4, NonZeroUsize::get);
        for _ in 0..threads_count {
            thread::spawn(thread_run);
        }
    });
}

fn thread_run() {
    loop {
        let mut queue = QUEUE.lock();
        if queue.is_empty() {
            CV.wait(&mut queue);
        }
        let task = queue.pop_front();
        drop(queue);
        if let Some(task) = task {
            task();
        }
    }
}

#[test]
fn run_tasks() {
    const COUNT: usize = 20;
    #[expect(clippy::needless_collect, reason = "must iterate all before join")]
    let handles: Vec<_> = (0..COUNT).map(|i| spawn(move || i)).collect();
    let expected = {
        let mut expected = [0; COUNT];
        expected.iter_mut().enumerate().for_each(|(i, v)| *v = i);
        expected
    };
    let results: Vec<_> = handles.into_iter().map(JoinHandle::join).collect();
    assert_eq!(results, expected);
}

#[test]
#[should_panic = "task that panicked"]
fn task_with_panic() {
    panic::set_hook(Box::new(|_| {}));
    spawn(|| panic!("task that panicked")).join();
}
