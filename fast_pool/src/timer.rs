use std::time::Duration;
use crate::task::PeriodicTask;
use crossbeam_channel::{unbounded, Sender, Receiver, RecvTimeoutError};

pub enum TimerAction {
    Schedule(PeriodicTask),
    Abort
}

enum RecvResult {
    Abort,
    Exit,
    Continue
}

pub struct Timer {
    tasks: Vec<PeriodicTask>,
    receiver: Receiver<TimerAction>,
    times: [u64; 34],
    sleep: u8
}

#[derive(Clone)]
pub struct TimerHandle {
    sender: Sender<TimerAction>
}

impl TimerHandle {
    pub fn schedule(&self, task: PeriodicTask) {
        let _ = self.sender.send(TimerAction::Schedule(task));
    }

    pub fn shutdown(self) {
        let _ = self.sender.send(TimerAction::Abort);
    }
}

impl TimerHandle {
    pub fn new() -> std::io::Result<Self> {
        Timer::init()
    }
}

impl Timer {
    pub fn init() -> std::io::Result<TimerHandle> {
        let (tx, rx) = unbounded();

        std::thread::Builder::new()
            .name("fast_pool-timer".to_string())
            .spawn(move || {
                Self {
                    tasks: Vec::new(),
                    receiver: rx,
                    times: [
                        100, 150, 200, 250, 300, 350, 400, 450, 500,
                        550, 600, 650, 700, 750, 800, 850, 900, 950,
                        1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800,
                        1900, 2000, 2100, 2200, 2300, 2400, 2500
                    ],
                    sleep: 0
                }.run();
            })?;

        Ok(TimerHandle {
            sender: tx
        })
    }

    fn try_recv_timeout(&mut self) -> RecvResult {
        if self.sleep == self.times.len() as u8 {
            self.sleep = 0;
            return RecvResult::Exit;
        }

        match self.receiver.recv_timeout(Duration::from_millis(self.times[self.sleep as usize])) {
            Ok(action) => {
                match action {
                    TimerAction::Schedule(task) => {
                        self.tasks.push(task);
                        self.sleep = 0;
                    },
                    TimerAction::Abort => return RecvResult::Abort
                }
            },
            Err(err) if matches!(err, RecvTimeoutError::Disconnected) => return RecvResult::Abort,
            _ => {
                self.sleep += 1;
            }
        }

        RecvResult::Continue
    }

    fn schedule_available(&mut self) {
        for task in self.tasks.drain_filter(|task| task.can_run()) {
            task.schedule();
        }
    }

    fn run(mut self) {
        loop {
            self.schedule_available();

            match self.try_recv_timeout() {
                RecvResult::Abort => break,
                RecvResult::Exit if self.tasks.is_empty() => break,
                _ => ()
            }
        }

        crate::context::delete_timer();
    }
}
