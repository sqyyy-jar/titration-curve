use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use anyhow::Result;

/// ## Possible states
///
/// - Idle
///   - With file
///   - Without file
/// - Busy
///   - With file
///   - Without file
/// - Dead
///   - Regular (worker was requested to stop)
///   - By error
#[derive(Clone, Copy)]
pub enum State {
    Idle { with_file: bool },
    Busy { new_file: bool },
    Dead { by_error: bool },
}

impl State {
    /// Returns if the state is alive.
    pub fn is_alive(self) -> bool {
        matches!(self, Self::Idle { .. } | Self::Busy { .. })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// The worker should request a file dialog
    FileDialog,
    /// The worker should update the file
    Update,
    /// The worker should stop itself
    Stop,
}

impl Signal {
    /// Checks if the signal activates signal-skipping.
    pub fn is_lock(self) -> bool {
        matches!(self, Self::FileDialog | Self::Stop)
    }

    /// Checks if the signal should be skipped with a given lock.
    pub fn should_skip(self, lock: Self) -> bool {
        match (lock, self) {
            (Self::FileDialog, Self::Update) => true,
            (Self::Stop, _) => true,
            _ => false,
        }
    }
}

/// ## Flow
///
/// ```text
/// App --> Worker
///   signal(file_dialog)
///   signal(stop)
///
/// App <-- Worker
///   state
///   result
///
/// Worker --> Notify
///   watch
///   unwatch
///
/// Worker <-- Notify
///   signal(update)
/// ```
///
/// ## Signals
///
/// Signals are sent to the worker through a channel. They are processed in the order they were sent
/// in.
///
/// While sending a signal the `signal_lock` mutex must stay locked. This is to prevent a race
/// condition where an update signal is sent directly after a file dialog signal, in which case the
/// file would be loaded two times in a row.
///
/// If the worker is busy with a new file, update signals will be ignored.
///
/// ## State
///
/// ```text
/// new:
///   state = idle without file
///
/// signal(update):
///   state = busy with current file
///   ...
///   state = idle with file
///
/// signal(file_dialog):
///   state = busy with new file
///   ...
///   state = idle with file
///
/// signal(stop):
///   state = dead
/// ```
pub struct Worker {
    /// This mutex provides information about the state of the worker. It is used to check if the
    /// state is still alive.
    state: Mutex<State>,
    /// This lock is used to ensure that signals are sent in the correct order.
    ///
    /// The mutex remains locked while sending a signal.
    ///
    /// The value in the mutex represents the lock to be used for skipping signals.
    signal_lock: Mutex<Option<Signal>>,
    /// This sender is used to send signals to the worker.
    signals: Sender<Signal>,
    result: Mutex<Option<WorkerResult>>,
}

impl Worker {
    fn new() -> (Self, Receiver<Signal>) {
        let (send, receiver) = channel();
        (
            Self {
                state: Mutex::new(State::Idle { with_file: false }),
                signal_lock: Mutex::default(),
                signals: send,
                result: Mutex::new(None),
            },
            receiver,
        )
    }

    /// Spawns a new worker.
    pub fn spawn() -> Arc<Self> {
        let (worker, receiver) = Self::new();
        let worker = Arc::new(worker);
        {
            let worker = worker.clone();
            thread::spawn(move || worker_impl(worker, receiver));
        }
        worker
    }

    /// Gets the workers state.
    pub fn get_state(&self) -> State {
        *self.state.lock().unwrap()
    }

    /// Sets the workers state.
    pub fn set_state(&self, state: State) {
        *self.state.lock().unwrap() = state;
    }

    /// Sends a signal to the worker.
    ///
    /// The function ensures that all signals are sent in the correct order.
    ///
    /// Additionally certain signals may be skipped.
    pub fn send_signal(&self, signal: Signal) {
        let mut lock = self.signal_lock.lock().unwrap();
        if let Some(lock) = *lock {
            if signal.should_skip(lock) {
                return;
            }
        }
        // This will automatically promote lower locks to higher ones (e.g. `FileDialog` ->
        // `Stop`).
        if signal.is_lock() {
            *lock = Some(signal);
        }
        self.signals.send(signal).unwrap();
        drop(lock);
    }

    /// Returns the worker result, if present, and removes it from the worker
    pub fn result(&self) -> Option<WorkerResult> {
        self.result.lock().unwrap().take()
    }
}

pub struct Input {}

pub enum WorkerResult {
    New(Arc<Output>),
    Update(Arc<Output>),
    NewError(WorkerError),
    UpdateError(WorkerError),
}

pub enum WorkerError {}

pub struct Output {
    pub items: Vec<OutputItem>,
}

pub struct OutputItem {
    pub ph: f64,
    pub volume: f64,
}

fn worker_impl(worker: Arc<Worker>, signals: Receiver<Signal>) {
    let Err(err) = worker_impl_try(worker.clone(), signals) else {
        worker.set_state(State::Dead { by_error: false });
        return;
    };
    eprintln!("[worker] The worker crashed: {err}");
    worker.set_state(State::Dead { by_error: true });
}

fn worker_impl_try(worker: Arc<Worker>, signals: Receiver<Signal>) -> Result<()> {
    let mut _watcher = notify::recommended_watcher(|_res| todo!())?;
    let mut _path: Option<PathBuf> = None;
    // watcher.watch(PathBuf::new().as_path(), RecursiveMode::NonRecursive);
    loop {
        let signal = signals.recv()?;
        match signal {
            Signal::FileDialog => {
                worker.set_state(State::Busy { new_file: true });
                let file = rfd::FileDialog::new().pick_file();
                worker.set_state(State::Idle { with_file: true });
            }
            Signal::Update => todo!(),
            Signal::Stop => break,
        }
    }
    Ok(())
}
