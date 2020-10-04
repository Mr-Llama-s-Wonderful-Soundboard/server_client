pub use server_client_proc_macro::server_client;
use crossbeam_channel::{Sender, Receiver, unbounded, SendError, RecvError, TryRecvError};

pub struct DuplexEnd<S, R = S> {
    s: Sender<S>,
    r: Receiver<R>,
}

impl<S, R> DuplexEnd<S, R> {
    pub fn send(&self, m: S) -> Result<(), SendError<S>> {
        self.s.send(m)
    }

    pub fn recv(&self) -> Result<R, RecvError> {
        self.r.recv()
    }

    pub fn try_recv(&self) -> Result<R, TryRecvError> {
        self.r.try_recv()
    }
}

pub fn duplex<T, U>() -> (DuplexEnd<T, U>, DuplexEnd<U, T>) {
    let (s1, r1) = unbounded();
    let (s2, r2) = unbounded();
    (DuplexEnd { s: s1, r: r2 }, DuplexEnd { s: s2, r: r1 })
}
