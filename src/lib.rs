pub use crossbeam_channel as channel;
use crossbeam_channel::{unbounded, Receiver, RecvError, SendError, Sender, TryRecvError};
pub use server_client_proc_macro::{server_client, encapsulate};

#[cfg(feature = "async")]
pub mod asynchronous {
    #[cfg(feature = "libtokio3")]
    pub use libtokio3::sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender, unbounded_channel as unbounded, error::{SendError, TryRecvError}};
    
    #[cfg(feature = "libtokio2")]
    pub use libtokio2::sync::mpsc::{UnboundedReceiver as Receiver, UnboundedSender as Sender, unbounded_channel as unbounded, error::{SendError, TryRecvError}};

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
    
        pub fn try_recv(&mut self) -> Result<R, TryRecvError> {
            self.r.try_recv()
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
