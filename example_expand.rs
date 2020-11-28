#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
fn main() {}
use server_client::server_client;
mod example_mod {
    use super::*;
    enum Request {
        NewConnection,
        DropConnection,
        Increment(usize),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Request {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Request::NewConnection,) => {
                    let mut debug_trait_builder = f.debug_tuple("NewConnection");
                    debug_trait_builder.finish()
                }
                (&Request::DropConnection,) => {
                    let mut debug_trait_builder = f.debug_tuple("DropConnection");
                    debug_trait_builder.finish()
                }
                (&Request::Increment(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Increment");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    enum Reply {
        NewConnection(ExampleClient),
        Increment(usize),
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for Reply {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match (&*self,) {
                (&Reply::NewConnection(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("NewConnection");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
                (&Reply::Increment(ref __self_0),) => {
                    let mut debug_trait_builder = f.debug_tuple("Increment");
                    let _ = debug_trait_builder.field(&&(*__self_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    pub struct ExampleServer {
        connections: Vec<::server_client::asynchronous::DuplexEnd<Reply, Request>>,
        a: usize,
    }
    impl ExampleServer {
        pub fn new(a: usize) -> Self {
            Self {
                connections: Vec::new(),
                a,
            }
        }
        pub fn connection(&mut self) -> ExampleClient {
            {
                ::std::io::_print(::core::fmt::Arguments::new_v1(
                    &["Connection #", "\n"],
                    &match (&self.connections.len(),) {
                        (arg0,) => [::core::fmt::ArgumentV1::new(
                            arg0,
                            ::core::fmt::Display::fmt,
                        )],
                    },
                ));
            };
            let (x1, x2) = ::server_client::asynchronous::duplex();
            self.connections.push(x1);
            ExampleClient::new(x2)
        }
        pub fn tick(&mut self) {
            for i in 0..self.connections.len() {
                if let Ok(m) = self.connections[i].try_recv() {
                    let (res, remove) = self.handle(m);
                    if let Some(r) = res {
                        self.connections[i].send(r).unwrap();
                    }
                    if remove {
                        {
                            ::std::io::_print(::core::fmt::Arguments::new_v1(
                                &["Dropped connection #", " of ", "\n"],
                                &match (&i, &(self.connections.len() - 1)) {
                                    (arg0, arg1) => [
                                        ::core::fmt::ArgumentV1::new(
                                            arg0,
                                            ::core::fmt::Display::fmt,
                                        ),
                                        ::core::fmt::ArgumentV1::new(
                                            arg1,
                                            ::core::fmt::Display::fmt,
                                        ),
                                    ],
                                },
                            ));
                        };
                        self.connections.remove(i);
                        break;
                    }
                }
            }
        }
        fn handle(&mut self, req: Request) -> (Option<Reply>, bool) {
            match req {
                Request::NewConnection => (Some(Reply::NewConnection(self.connection())), false),
                Request::DropConnection => (None, true),
                Request::Increment(b) => (
                    Some(Reply::Increment({
                        self.a += b;
                        self.a
                    })),
                    false,
                ),
            }
        }
    }
    pub struct ExampleClient {
        connection: ::server_client::asynchronous::DuplexEnd<Request, Reply>,
    }
    impl ExampleClient {
        fn new(connection: ::server_client::asynchronous::DuplexEnd<Request, Reply>) -> Self {
            Self { connection }
        }
        pub fn increment(&self, b: usize) -> usize {
            self.connection.send(Request::Increment(b)).unwrap();
            if let Reply::Increment(x) = self.connection.recv().unwrap() {
                x
            } else {
                {
                    {
                        {
                            :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "internal error: entered unreachable code: " ] , & match ( & "Unexpected non return for fn (is this client being used in parallel?)" , ) { ( arg0 , ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) ] , } ) )
                        }
                    }
                };
            }
        }
        pub async fn clone(&mut self) -> Self {
            self.connection.send(Request::NewConnection).unwrap();
            if let Reply::NewConnection(x) = self.connection.recv().await.unwrap() {
                x
            } else {
                {
                    {
                        {
                            :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "internal error: entered unreachable code: " ] , & match ( & "Unexpected non return for fn (is this client being used in parallel?)" , ) { ( arg0 , ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) ] , } ) )
                        }
                    }
                };
            }
        }
    }
    impl std::fmt::Debug for ExampleClient {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(::core::fmt::Arguments::new_v1(
                &["#client_name"],
                &match () {
                    () => [],
                },
            ))
        }
    }
    impl Drop for ExampleClient {
        fn drop(&mut self) {
            self.connection.send(Request::DropConnection).unwrap();
        }
    }
}
pub use example_mod::{ExampleServer, ExampleClient};
