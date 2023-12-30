use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use calamine::Reader;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use rfd::FileDialog;

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
pub struct Worker {
    /// This flag is true as long as the worker is alive.
    alive: Mutex<bool>,
    /// This lock is used to ensure that signals are sent in the correct order.
    ///
    /// The mutex remains locked while sending a signal.
    ///
    /// The value in the mutex represents the lock to be used for skipping signals.
    signal_lock: Mutex<Option<Signal>>,
    /// This sender is used to send signals to the worker.
    signal_sender: Sender<Signal>,
    /// This sender is used to send responses to the app.
    response_sender: Sender<Response>,
}

impl Worker {
    fn new() -> (Self, Receiver<Signal>, Receiver<Response>) {
        let (signal_sender, signal_receiver) = channel();
        let (response_sender, response_receiver) = channel();
        (
            Self {
                alive: Mutex::new(true),
                signal_lock: Mutex::default(),
                signal_sender,
                response_sender,
            },
            signal_receiver,
            response_receiver,
        )
    }

    /// Spawns a new worker.
    pub fn spawn() -> (Arc<Self>, Receiver<Response>) {
        let (worker, signal_receiver, response_receiver) = Self::new();
        let worker = Arc::new(worker);
        {
            let worker = worker.clone();
            thread::Builder::new()
                .name("worker".into())
                .spawn(move || worker_impl(worker, signal_receiver))
                .expect("spawn worker thread");
        }
        (worker, response_receiver)
    }

    /// Checks if the worker is alive.
    pub fn is_alive(&self) -> bool {
        *self.alive.lock().unwrap()
    }

    /// Sets if the worker is alive.
    ///
    /// This function is meant to be used by the worker itself.
    ///
    /// The worker should be stopped by a signal.
    pub fn set_alive(&self, alive: bool) {
        *self.alive.lock().unwrap() = alive;
    }

    /// Resets the flag introduced by the given signal.
    pub fn reset_signal_lock(&self, signal: Signal) {
        let mut lock = self.signal_lock.lock().unwrap();
        let Some(lock_signal) = *lock else {
            return;
        };
        // Higher locks cannot be unlocked by lower locks.
        if !lock_signal.can_unlock(signal) {
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
        _ = self.signal_sender.send(signal);
        drop(lock);
    }

    /// Sends a response to the app.
    pub fn send_response(&self, response: Response) {
        _ = self.response_sender.send(response);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Signal {
    /// The worker should update the file.
    Update = 0,
    /// The worker should request a file dialog.
    FileDialog = 1,
    /// The worker should stop itself.
    Stop = 2,
}

impl Signal {
    /// Checks if the signal activates signal-skipping.
    pub fn is_lock(self) -> bool {
        // matches!(self, Self::FileDialog | Self::Update | Self::Stop)
        true
    }

    /// Checks if the signal should be skipped with a given lock.
    pub fn should_skip(self, lock: Self) -> bool {
        match lock {
            Self::Update => self <= lock,
            Self::FileDialog => self <= lock,
            Self::Stop => true,
        }
    }

    pub fn can_unlock(self, lock: Self) -> bool {
        match lock {
            Signal::Update => self >= lock,
            Signal::FileDialog => self >= lock,
            Signal::Stop => false,
        }
    }
}

#[derive(Debug)]
pub enum Response {
    /// The current file should be unloaded.
    Unload,
    Output(Arc<Output>),
    Error(WorkerError),
}

#[derive(Debug)]
pub enum WorkerError {
    FileDoesNotExist,
    TableError(calamine::Error),
    NoTableInWorkbook,
    TableNotCorrectlyFormatted,
}

impl Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

fn worker_impl(worker: Arc<Worker>, signal_receiver: Receiver<Signal>) {
    if let Err(err) = worker_impl_try(worker.clone(), signal_receiver) {
        eprintln!("[worker] The worker crashed: {err}");
    }
    worker.set_alive(false);
    eprintln!("[worker] The worker shut down");
}

fn worker_impl_try(worker: Arc<Worker>, signal_receiver: Receiver<Signal>) -> Result<()> {
    let mut path: Option<PathBuf> = None;
    let mut watcher = {
        let worker = worker.clone();
        // INotifyWatcher does not work
        notify::PollWatcher::new(
            move |res| {
                let event: Event = match res {
                    Ok(event) => event,
                    Err(err) => {
                        eprintln!("[watcher] There was an error during the event stream: {err}");
                        return;
                    }
                };
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Remove(_)) {
                    worker.send_signal(Signal::Update);
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(500)),
        )?
    };
    loop {
        let signal = signal_receiver.recv()?;
        match signal {
            Signal::FileDialog => 'blk: {
                let Some(file) = FileDialog::new()
                    .add_filter(
                        "Tabelle",
                        &["xls", "xlsx", "xlsm", "xlsb", "xla", "xlam", "ods"],
                    )
                    .pick_file()
                else {
                    break 'blk;
                };
                if !file.is_file() {
                    worker.send_response(Response::Error(WorkerError::FileDoesNotExist));
                    break 'blk;
                }
                watcher.watch(&file, RecursiveMode::NonRecursive)?;
                load_file(&worker, &file);
                path = Some(file);
            }
            Signal::Update => 'blk: {
                let Some(some_path) = &path else {
                    break 'blk;
                };
                if !some_path.is_file() {
                    _ = watcher.unwatch(some_path);
                    path = None;
                    worker.send_response(Response::Unload);
                    break 'blk;
                }
                load_file(&worker, some_path);
            }
            Signal::Stop => break,
        }
        worker.reset_signal_lock(signal);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Input {
    pub t_v: f32,
    pub t_c: f32,
    pub m_c: f32,
    pub m_v: Vec<f32>,
    pub acid: f32,
    pub base: f32,
}

impl Input {
    pub fn calculate_output(&self) -> Output {
        let mut items = Vec::new();
        // todo
        for &m_v in &self.m_v {
            let total_v = m_v + 10.0;
            let n1 = 0.001;
            let n2 = m_v / 1000.0 * self.m_c;
            let c1 = n1 / (total_v / 1000.0);
            let c2 = 0.0;
            let ph = -c1.log10();
            let poh = 14.0 - ph;
            items.push(OutputItem {
                m_v,
                ph,
                total_v,
                n1,
                n2,
                c1,
                c2,
                poh,
            });
        }
        Output { items }
    }
}

#[derive(Debug)]
pub struct Output {
    pub items: Vec<OutputItem>,
}

impl Output {
    pub fn max_m_v(&self) -> f32 {
        self.items
            .iter()
            .map(|it| it.m_v)
            .reduce(f32::max)
            .unwrap_or(0.0)
    }
}

#[derive(Debug)]
pub struct OutputItem {
    pub m_v: f32,
    pub ph: f32,
    pub total_v: f32,
    pub n1: f32,
    pub n2: f32,
    pub c1: f32,
    pub c2: f32,
    pub poh: f32,
}

/// Loads a file from the given path.
///
/// The table format is the following:
///
/// ```text
/// t: test solution
/// m: measuring solution
/// V: volume
/// c: concentration
/// acid: acid used for titration
/// base: base used for titration
///
/// +--------+----+-------+----+----+------+
/// |        |    | V (t) |    |    | acid |
/// +--------+----+-------+----+----+------+
/// |        |    | c (t) |    |    |      |
/// +--------+----+-------+----+----+------+
/// |        |    | c (m) |    |    | base |
/// +--------+----+-------+----+----+------+
/// |        |    |       |    |    |      |
/// +--------+----+-------+----+----+------+
/// |        |    |       |    |    |      |
/// +--------+----+-------+----+----+------+
/// | V0 (m) |    |       |    |    |      |
/// +--------+----+-------+----+----+------+
/// | V1 (m) |    |       |    |    |      |
/// +--------+----+-------+----+----+------+
/// | ...    |    |       |    |    |      |
/// +--------+----+-------+----+----+------+
/// ```
fn load_file(worker: &Worker, path: &PathBuf) {
    let mut workbook = match calamine::open_workbook_auto(path) {
        Ok(workbook) => workbook,
        Err(err) => {
            worker.send_response(Response::Error(WorkerError::TableError(err)));
            return;
        }
    };
    let Some(worksheet) = workbook.worksheet_range_at(0) else {
        worker.send_response(Response::Error(WorkerError::NoTableInWorkbook));
        return;
    };
    let worksheet = match worksheet {
        Ok(worksheet) => worksheet,
        Err(err) => {
            worker.send_response(Response::Error(WorkerError::TableError(err)));
            return;
        }
    };
    let (h, w) = worksheet.get_size();
    if h < 6 || w < 6 {
        worker.send_response(Response::Error(WorkerError::TableNotCorrectlyFormatted));
        return;
    }
    let (Some(t_v), Some(t_c), Some(m_c)) = (
        worksheet[(0, 2)].as_f64(),
        worksheet[(1, 2)].as_f64(),
        worksheet[(2, 2)].as_f64(),
    ) else {
        worker.send_response(Response::Error(WorkerError::TableNotCorrectlyFormatted));
        return;
    };
    let mut m_v = Vec::new();
    for row in worksheet.rows().skip(5) {
        if row.is_empty() {
            worker.send_response(Response::Error(WorkerError::TableNotCorrectlyFormatted));
            return;
        }
        let Some(cell) = row[0].as_f64() else {
            worker.send_response(Response::Error(WorkerError::TableNotCorrectlyFormatted));
            return;
        };
        m_v.push(cell as f32);
    }
    // todo
    let input = Input {
        t_v: t_v as f32,
        t_c: t_c as f32,
        m_c: m_c as f32,
        m_v,
        acid: 0.0,
        base: 0.0,
    };
    let output = input.calculate_output();
    worker.send_response(Response::Output(Arc::new(output)));
}
