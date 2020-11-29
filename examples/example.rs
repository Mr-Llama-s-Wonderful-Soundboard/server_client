fn main() {
	let mut server = AServer::new(A);
	let conn = server.connection();
}

use server_client::{server_client, encapsulate};

// server_client!(
// 	pub Example {
// 		let a: usize

// 		fn increment(b: usize) -> usize {
// 			self.a += b;
// 			self.a
// 		}
// 	}
// );


pub struct A;

#[encapsulate(ordered)]
impl A {
	pub fn hah() {

	}
}

// pub struct A;
// impl A {}
// mod a_mod {
//     use super::*;
//     enum Request {
//         NewConnection,
//         DropConnection,
//     }
//     #[automatically_derived]
//     #[allow(unused_qualifications)]
//     impl ::core::fmt::Debug for Request {
//         fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//             match (&*self,) {
//                 (&Request::NewConnection,) => {
//                     let mut debug_trait_builder = f.debug_tuple("NewConnection");
//                     debug_trait_builder.finish()
//                 }
//                 (&Request::DropConnection,) => {
//                     let mut debug_trait_builder = f.debug_tuple("DropConnection");
//                     debug_trait_builder.finish()
//                 }
//             }
//         }
//     }
//     enum Reply {
//         NewConnection(AClient),
//     }
//     #[automatically_derived]
//     #[allow(unused_qualifications)]
//     impl ::core::fmt::Debug for Reply {
//         fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
//             match (&*self,) {
//                 (&Reply::NewConnection(ref __self_0),) => {
//                     let mut debug_trait_builder = f.debug_tuple("NewConnection");
//                     let _ = debug_trait_builder.field(&&(*__self_0));
//                     debug_trait_builder.finish()
//                 }
//             }
//         }
//     }
//     pub struct AServer {
//         connections: ::server_client::IdMap<::server_client::channel::Sender<Reply>>,
//         receiver: ::server_client::channel::Receiver<(Request, usize)>,
//         sender: ::server_client::channel::Sender<(Request, usize)>,
//         value: A,
//     }
//     impl AServer {
//         pub fn new(value: A) -> Self {
//             let (sender, receiver) = ::server_client::channel::unbounded();
//             Self {
//                 connections: ::server_client::IdMap::new(),
//                 receiver,
//                 sender,
//                 value,
//             }
//         }
//         pub fn connection(&mut self) -> AClient {
//             let (tx, rx) = ::server_client::channel::unbounded();
//             let id = self.connections.add(tx);
//             AClient::new(rx, self.sender.clone(), id)
//         }
//         pub fn tick(&mut self) {
//             while let Ok((req, id)) = self.receiver.try_recv() {
//                 let (res, remove) = self.handle(req);
//                 if let Some(r) = res {
//                     self.connections[id].send(r).unwrap();
//                 }
//                 if remove {
//                     self.connections.remove(id);
//                     break;
//                 }
//             }
//         }
//         fn handle(&mut self, req: Request) -> (Option<Reply>, bool) {
//             match req {
//                 Request::NewConnection => (Some(Reply::NewConnection(self.connection())), false),
//                 Request::DropConnection => (None, true),
//             }
//         }
//     }
//     pub struct AClient {
//         receiver: ::server_client::channel::Receiver<Reply>,
//         sender: ::server_client::channel::Sender<(Request, usize)>,
//         id: usize,
//     }
//     impl AClient {
//         fn new(
//             receiver: ::server_client::channel::Receiver<Reply>,
//             sender: ::server_client::channel::Sender<(Request, usize)>,
//             id: usize,
//         ) -> Self {
//             Self {
//                 receiver,
//                 sender,
//                 id,
//             }
//         }
//     }
//     impl Clone for AClient {
//         fn clone(&self) -> Self {
//             self.sender.send((Request::NewConnection, self.id)).unwrap();
//             if let Reply::NewConnection(x) = self.receiver.recv().unwrap() {
//                 x
//             } else {
//                 // {
//                 //     {
//                 //         {
//                 //             :: std :: rt :: begin_panic_fmt ( & :: core :: fmt :: Arguments :: new_v1 ( & [ "internal error: entered unreachable code: " ] , & match ( & "Unexpected non return for fn (is this client being used in parallel?)" , ) { ( arg0 , ) => [ :: core :: fmt :: ArgumentV1 :: new ( arg0 , :: core :: fmt :: Display :: fmt ) ] , } ) )
//                 //         }
//                 //     }
// 				// };
// 				unreachable!()
//             }
//         }
//     }
//     impl std::fmt::Debug for AClient {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             // f.write_fmt(::core::fmt::Arguments::new_v1(
//             //     &["#client_name[", "]"],
//             //     &match (&self.id,) {
//             //         (arg0,) => [::core::fmt::ArgumentV1::new(
//             //             arg0,
//             //             ::core::fmt::Display::fmt,
//             //         )],
//             //     },
// 			// ))
// 			todo!()
//         }
//     }
//     impl Drop for AClient {
//         fn drop(&mut self) {
//             self.sender
//                 .send((Request::DropConnection, self.id))
//                 .unwrap();
//         }
//     }
// }
// use a_mod::{AServer, AClient};