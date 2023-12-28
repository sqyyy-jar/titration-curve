use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use anyhow::Result;
use notify::{Event, EventKind, Watcher};

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
    signal_sender: Sender<Signal>,
}

impl Worker {
    fn new() -> (Self, Receiver<Signal>, Sender<Response>, Receiver<Response>) {
        let (signal_sender, signal_receiver) = channel();
        let (response_sender, response_receiver) = channel();
        (
            Self {
                state: Mutex::new(State::Idle { with_file: false }),
                signal_lock: Mutex::default(),
                signal_sender,
            },
            signal_receiver,
            response_sender,
            response_receiver,
        )
    }

    /// Spawns a new worker.
    pub fn spawn() -> (Arc<Self>, Receiver<Response>) {
        let (worker, signal_receiver, response_sender, response_receiver) = Self::new();
        let worker = Arc::new(worker);
        {
            let worker = worker.clone();
            thread::spawn(move || worker_impl(worker, signal_receiver, response_sender));
        }
        (worker, response_receiver)
    }

    /// Gets the workers state.
    pub fn get_state(&self) -> State {
        *self.state.lock().unwrap()
    }

    /// Sets the workers state.
    pub fn set_state(&self, state: State) {
        *self.state.lock().unwrap() = state;
    }

    /// Resets the flag introduced by the given signal.
    pub fn reset_signal_lock(&self, signal: Signal) {
        let mut lock = self.signal_lock.lock().unwrap();
        let Some(lock_signal) = *lock else {
            return;
        };
        // Higher locks cannot be unlocked by lower locks.
        if lock_signal.should_skip(signal) {
            return;
        }
        *lock = None;
    }

    /// Sends a signal to the worker.
    ///
    /// The function ensures that all signals are sent in the correct order.
    ///
    /// Additionally certain signals may be skipped, depending on an internal lock.
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
        self.signal_sender.send(signal).unwrap();
        drop(lock);
    }
}

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
    /// The worker should request a file dialog.
    FileDialog,
    /// The worker should update the file.
    Update,
    /// The worker should unload the file and notify the app.
    Unload,
    /// The worker should stop itself.
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

pub enum Response {
    Output(Arc<Output>),
    Error(WorkerError),
}

pub enum WorkerError {
    WatcherError(Box<notify::Error>),
}

pub struct Input {}

pub struct Output {
    pub items: Vec<OutputItem>,
}

pub struct OutputItem {
    pub ph: f64,
    pub volume: f64,
}

fn worker_impl(
    worker: Arc<Worker>,
    signal_receiver: Receiver<Signal>,
    response_sender: Sender<Response>,
) {
    let Err(err) = worker_impl_try(worker.clone(), signal_receiver, response_sender) else {
        worker.set_state(State::Dead { by_error: false });
        return;
    };
    eprintln!("[worker] The worker crashed: {err}");
    worker.set_state(State::Dead { by_error: true });
}

fn worker_impl_try(
    worker: Arc<Worker>,
    signal_receiver: Receiver<Signal>,
    response_sender: Sender<Response>,
) -> Result<()> {
    let mut watcher = notify::recommended_watcher(move |res| {
        // huh? rustc, go fix yourself
        let event: Event = match res {
            Ok(event) => event,
            Err(err) => {
                _ = response_sender.send(Response::Error(WorkerError::WatcherError(Box::new(err))));
                return;
            }
        };
        if !matches!(event.kind, EventKind::Modify(_) | EventKind::Remove(_)) {
            return;
        }
        todo!("Try to reload file or send unload signal if not present")
    })?;
    let mut path: Option<PathBuf> = None;
    // watcher.watch(PathBuf::new().as_path(), RecursiveMode::NonRecursive);
    loop {
        let signal = signal_receiver.recv()?;
        match signal {
            Signal::FileDialog => {
                worker.set_state(State::Busy { new_file: true });
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    todo!("Implement FileDialog")
                }
                worker.set_state(State::Idle { with_file: true });
            }
            Signal::Update => todo!("Implement Update"),
            Signal::Unload => {
                if let Some(path) = &path {
                    watcher.unwatch(path)?;
                }
                path = None;
                todo!("Implement Unload")
            }
            Signal::Stop => break,
        }
        worker.reset_signal_lock(signal);
    }
    Ok(())
}
