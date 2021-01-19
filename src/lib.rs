pub use crossbeam_channel as channel;
use crossbeam_channel::{unbounded, Receiver, RecvError, SendError, Sender, TryRecvError};
pub use server_client_proc_macro::{encapsulate, server_client};

#[cfg(feature = "async")]
pub mod asynchronous {

    #[cfg(feature = "libtokio1")]
    pub use libtokio1::sync::mpsc::{
        error::SendError, unbounded_channel as tokio_unbounded, UnboundedReceiver as TokioReceiver,
        UnboundedSender as Sender,
    };

    #[cfg(feature = "libtokio03")]
    pub use libtokio03::sync::mpsc::{
        error::{SendError, TryRecvError as TokioTryRecvError},
        unbounded_channel as tokio_unbounded, UnboundedReceiver as TokioReceiver, UnboundedSender as Sender,
    };

    #[cfg(feature = "libtokio02")]
    pub use libtokio02::sync::mpsc::{
        error::{SendError, TryRecvError as TokioTryRecvError},
        unbounded_channel as tokio_unbounded, UnboundedReceiver as TokioReceiver, UnboundedSender as Sender,
    };

    pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
        let (s, r) = tokio_unbounded();
        (s, Receiver::new(r))
    }

    pub enum TryRecvError {
        Empty,
        Closed,
    }

    #[cfg(any(feature = "libtokio02", feature = "libtokio03"))]
    impl From<TokioTryRecvError> for TryRecvError {
        fn from(error: TokioTryRecvError) -> Self {
            match error {
                TokioTryRecvError::Closed => Self::Closed,
                TokioTryRecvError::Empty => Self::Empty
            }
        }
    }

    pub struct Receiver<T> {
        receiver: TokioReceiver<T>
    }

    
    impl<T> Receiver<T> {
        pub fn new(receiver: TokioReceiver<T>) -> Self { Self { receiver } }


        pub async fn recv(&mut self) -> Option<T> {
            self.receiver.recv().await
        }

        #[cfg(feature = "libtokio1")]
        pub async fn try_recv(&mut self) -> Result<T, TryRecvError> {
            let sleep = libtokio1::time::sleep(std::time::Duration::from_nanos(1));
            libtokio1::select! {
                _ = sleep => {
                    Err(TryRecvError::Empty)
                },
                x = self.receiver.recv() => {
                    match x {
                        Some(x) => Ok(x),
                        None => Err(TryRecvError::Closed)
                    }
                }
            } // Recv has to return before 1ns
        }

        #[cfg(any(feature = "libtokio02", feature = "libtokio03"))]
        pub async fn try_recv(&mut self) -> Result<T, TryRecvError> {
            Ok(self.r.try_recv().await?)
        }
    }

    pub struct DuplexEnd<S, R = S> {
        s: Sender<S>,
        r: Receiver<R>,
    }

    impl<S, R> DuplexEnd<S, R> {
        pub fn send(&self, m: S) -> Result<(), SendError<S>> {
            self.s.send(m)
        }

        pub async fn recv(&mut self) -> Option<R> {
            self.r.recv().await
        }

        #[cfg(any(feature = "libtokio02", feature = "libtokio03", feature = "libtokio1"))]
        pub async fn try_recv(&mut self) -> Result<R, TryRecvError> {
            self.r.try_recv().await
        }
    }

    pub fn duplex<T, U>() -> (DuplexEnd<T, U>, DuplexEnd<U, T>) {
        let (s1, r1) = unbounded();
        let (s2, r2) = unbounded();
        (DuplexEnd { s: s1, r: r2 }, DuplexEnd { s: s2, r: r1 })
    }
}

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

use avl::AvlTreeSet;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

#[derive(Clone)]
pub struct IdMap<V> {
    v: HashMap<usize, V>,
    holes: AvlTreeSet<usize>,
    len: usize,
}

impl<V> IdMap<V> {
    pub fn new() -> Self {
        Self {
            v: HashMap::new(),
            holes: AvlTreeSet::new(),
            len: 0,
        }
    }

    pub fn add(&mut self, val: V) -> usize {
        if self.holes.is_empty() {
            self.v.insert(self.len, val);
            let r = self.len;
            self.len += 1;
            r
        } else {
            let r = *(&self.holes).into_iter().next().unwrap();
            self.holes.remove(&r);
            self.v.insert(r, val);
            r
        }
    }

    pub fn remove(&mut self, id: usize) -> Option<V> {
        let r = self.v.remove(&id);
        if r.is_some() {
            self.holes.remove(&id);
        }
        r
    }
}

impl<V> Index<usize> for IdMap<V> {
    type Output = V;
    fn index(&self, index: usize) -> &Self::Output {
        &self.v[&index]
    }
}

impl<V> IndexMut<usize> for IdMap<V> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.v.get_mut(&index).expect("Id not valid")
    }
}
