//! Task executor for async tasks.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};
use lazyinit::LazyInit;
use spin::Mutex;

/// Type alias for a pinned and boxed future.
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

// Global executor singleton
static GLOBAL_EXECUTOR: LazyInit<Executor> = LazyInit::new();

// Per-CPU executor for local tasks
#[percpu::def_percpu]
static CPU_LOCAL_EXECUTOR: RefCell<Option<Executor>> = RefCell::new(None);

/// Helper function to get the global executor, initializing it if needed.
pub fn executor() -> &'static Executor {
    if !GLOBAL_EXECUTOR.is_inited() {
        GLOBAL_EXECUTOR.init_once(Executor::new());
    }
    GLOBAL_EXECUTOR
        .get()
        .expect("IMPOSSIBLE: global executor not initialized")
}

/// Spawns a new asynchronous task on the global executor.
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    executor().spawn(future)
}

/// Initialize the global executor runtime.
pub fn init() {
    // Initialize the global executor if it hasn't been initialized yet
    if !GLOBAL_EXECUTOR.is_inited() {
        GLOBAL_EXECUTOR.init_once(Executor::new());
    }
}

/// Runs the global executor until all tasks are complete.
pub fn run() {
    executor().run();
}

/// Initialize the per-CPU local executor.
fn ensure_local_executor() -> &'static RefCell<Option<Executor>> {
    let cell = unsafe { CPU_LOCAL_EXECUTOR.current_ptr() };
    let cell_ref = unsafe { &*cell };
    if cell_ref.borrow().is_none() {
        *cell_ref.borrow_mut() = Some(Executor::new());
    }
    cell_ref
}

/// Spawns a future on the current CPU's local executor.
pub fn spawn_local<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let cell = ensure_local_executor();
    // Initialize the executor if needed
    if cell.borrow().is_none() {
        *cell.borrow_mut() = Some(Executor::new());
    }

    // Unwrap is safe because we just ensured the executor exists
    let executor = cell.borrow();
    let executor = executor.as_ref().unwrap();

    // We need to use raw pointers since we can't return a reference from the RefCell borrow
    let executor_ptr = executor as *const Executor;

    // SAFETY: We ensure the pointer is valid during the scope of this call
    unsafe { (*executor_ptr).spawn(future) }
}

/// Run the current CPU's local executor until completion.
pub fn run_local() {
    let cell = ensure_local_executor();
    if let Some(executor) = cell.borrow().as_ref() {
        // We need to use raw pointers since we can't modify through a shared reference
        let executor_ptr = executor as *const Executor;
        // SAFETY: We ensure the pointer is valid during the scope of this call
        unsafe { (*(executor_ptr as *mut Executor)).run() };
    }
}

/// An executor that can run futures to completion.
pub struct Executor {
    // Task queue
    ready_tasks: Mutex<VecDeque<Task>>,
}

impl Executor {
    /// Creates a new executor.
    pub fn new() -> Self {
        Self {
            ready_tasks: Mutex::new(VecDeque::new()),
        }
    }

    /// Adds a task to the executor's queue.
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (task, handle) = Task::new(future, self);
        self.ready_tasks.lock().push_back(task);
        handle
    }

    /// Runs the executor until all tasks are complete.
    pub fn run(&self) {
        while self.step() {}
    }

    /// Runs a single step of the executor.
    ///
    /// Returns `true` if there are still tasks in the queue.
    pub fn step(&self) -> bool {
        let mut ready_tasks = self.ready_tasks.lock();
        if let Some(mut task) = ready_tasks.pop_front() {
            // Create a waker and poll the task
            let waker = task.waker();
            let mut cx = Context::from_waker(&waker);

            let future = unsafe { Pin::new_unchecked(&mut task.future) };

            info!("poll");
            if future.poll(&mut cx).is_pending() {
                info!("poll2");
                // Task is still pending, only re-queue if it hasn't been manually queued
                if !task.was_woken {
                    ready_tasks.push_back(task);
                }
            }

            !ready_tasks.is_empty()
        } else {
            false
        }
    }

    // Queue a task, used by the waker
    fn queue_task(&self, task: Task) {
        self.ready_tasks.lock().push_back(task);
    }

    /// Blocks on a future until it completes, using this executor.
    pub fn block_on<F>(&self, mut fut: F) -> F::Output
    where
        F: Future,
    {
        // safety: we don't move the future after this line.
        let mut fut = unsafe { Pin::new_unchecked(&mut fut) };

        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        loop {
            // Poll the future
            if let Poll::Ready(res) = fut.as_mut().poll(&mut cx) {
                return res;
            }

            // Run a step of this executor to make progress on other tasks
            self.step();

            // If the future is still not ready, yield to other tasks
            if self.ready_tasks.lock().is_empty() {
                axtask::yield_now();
            }
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

// Task definition - boxed future
pub(crate) struct Task {
    future: BoxFuture<()>,
    executor: *const Executor,
    was_woken: bool,
}

// Tasks must be Send to be spawned on other threads
unsafe impl Send for Task {}

impl Task {
    fn new<F>(future: F, executor: &Executor) -> (Self, JoinHandle<F::Output>)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (output_sender, output_receiver) = channel::oneshot::channel();

        // Create a future that sends the output through the channel
        let future = async move {
            let output = future.await;
            let _ = output_sender.send(output);
        };

        let task = Task {
            future: Box::pin(future),
            executor: executor as *const _,
            was_woken: false,
        };

        let handle = JoinHandle {
            receiver: output_receiver,
        };

        (task, handle)
    }

    fn waker(&mut self) -> Waker {
        // SAFETY: We ensure the executor ptr always lives as long as the task
        let executor = unsafe { &*self.executor };

        // Create a waker that will queue this task in the executor
        TaskWaker {
            task: self,
            executor,
        }
        .into_waker()
    }
}

struct TaskWaker<'a> {
    task: *mut Task,
    executor: &'a Executor,
}

// TaskWaker must be Send+Sync to be used across threads
unsafe impl<'a> Send for TaskWaker<'a> {}
unsafe impl<'a> Sync for TaskWaker<'a> {}

impl<'a> TaskWaker<'a> {
    fn into_waker(self) -> Waker {
        use core::task::{RawWaker, RawWakerVTable};

        // Convert TaskWaker to raw pointer
        let ptr = Box::into_raw(Box::new(self)) as *const ();

        // Define vtable with wake, clone, etc. functions
        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            // Clone
            |ptr| {
                let original = unsafe { &*(ptr as *const TaskWaker) };
                let cloned = TaskWaker {
                    task: original.task,
                    executor: original.executor,
                };
                let ptr = Box::into_raw(Box::new(cloned)) as *const ();
                RawWaker::new(ptr, &VTABLE)
            },
            // Wake
            |ptr| {
                let waker = unsafe { Box::from_raw(ptr as *mut TaskWaker) };
                waker.wake_task();
            },
            // Wake by reference
            |ptr| {
                let waker = unsafe { &*(ptr as *const TaskWaker) };
                waker.wake_task_by_ref();
            },
            // Drop
            |ptr| {
                unsafe {
                    drop(Box::from_raw(ptr as *mut TaskWaker));
                };
            },
        );

        unsafe { Waker::from_raw(RawWaker::new(ptr, &VTABLE)) }
    }

    fn wake_task(self) {
        // Mark the task as woken and queue it for execution
        unsafe {
            (*self.task).was_woken = true;

            // Create a clone of the task to queue
            let future = core::ptr::read(&(*self.task).future);
            let task = Task {
                future,
                executor: (*self.task).executor,
                was_woken: true,
            };

            self.executor.queue_task(task);
        }
    }

    fn wake_task_by_ref(&self) {
        unsafe {
            if !(*self.task).was_woken {
                (*self.task).was_woken = true;

                // Create a clone of the task to queue
                let future = core::ptr::read(&(*self.task).future);
                let task = Task {
                    future,
                    executor: (*self.task).executor,
                    was_woken: true,
                };

                self.executor.queue_task(task);
            }
        }
    }
}

/// A handle to a spawned task.
pub struct JoinHandle<T> {
    receiver: channel::oneshot::Receiver<T>,
}

impl<T: Send + 'static> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.poll(cx) {
            Poll::Ready(Ok(value)) => Poll::Ready(value),
            Poll::Ready(Err(_)) => panic!("Task failed to complete"),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A simple oneshot channel implementation.
pub mod channel {
    pub mod oneshot {
        use alloc::sync::Arc;
        use core::cell::UnsafeCell;
        use core::future::Future;
        use core::pin::Pin;
        use core::sync::atomic::{AtomicBool, Ordering};
        use core::task::{Context, Poll, Waker};
        use spin::Mutex;

        pub struct Sender<T> {
            inner: Arc<Inner<T>>,
        }

        pub struct Receiver<T> {
            inner: Arc<Inner<T>>,
        }

        struct Inner<T> {
            value: UnsafeCell<Option<T>>,
            complete: AtomicBool,
            waker: Mutex<Option<Waker>>,
        }

        // Safety: The channel ensures proper synchronization for the value.
        unsafe impl<T: Send> Send for Inner<T> {}
        unsafe impl<T: Send> Sync for Inner<T> {}

        pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
            let inner = Arc::new(Inner {
                value: UnsafeCell::new(None),
                complete: AtomicBool::new(false),
                waker: Mutex::new(None),
            });

            let sender = Sender {
                inner: inner.clone(),
            };

            let receiver = Receiver { inner };

            (sender, receiver)
        }

        impl<T> Sender<T> {
            pub fn send(self, value: T) -> Result<(), T> {
                if self.inner.complete.load(Ordering::Acquire) {
                    return Err(value);
                }

                unsafe {
                    *self.inner.value.get() = Some(value);
                }

                self.inner.complete.store(true, Ordering::Release);

                if let Some(waker) = self.inner.waker.lock().take() {
                    waker.wake();
                }

                Ok(())
            }
        }

        impl<T> Receiver<T> {
            pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<T, ()>> {
                if self.inner.complete.load(Ordering::Acquire) {
                    let value = unsafe { (*self.inner.value.get()).take() };
                    Poll::Ready(Ok(value.unwrap()))
                } else {
                    *self.inner.waker.lock() = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }

        impl<T> Future for Receiver<T> {
            type Output = Result<T, ()>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                trace!("oneshot poll");
                Pin::get_mut(self).poll(cx)
            }
        }

        impl<T> Drop for Inner<T> {
            fn drop(&mut self) {
                // Ensure the value is dropped
                unsafe {
                    let _ = *self.value.get();
                }
            }
        }
    }
}

/// Blocks on a future until it completes, using the global executor.
pub fn block_on<F>(fut: F) -> F::Output
where
    F: Future,
{
    executor().block_on(fut)
}

/// Creates a new [`Waker`] that is a no-op.
pub fn dummy_waker() -> Waker {
    use core::task::{RawWaker, RawWakerVTable};

    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        |_| RawWaker::new(core::ptr::null(), &VTABLE),
        |_| {},
        |_| {},
        |_| {},
    );
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}

/// Polls a future once, returning `Poll::Ready` if it completes.
pub fn poll_once<F>(fut: &mut F) -> Poll<F::Output>
where
    F: Future,
{
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    unsafe { Pin::new_unchecked(fut) }.poll(&mut cx)
}
