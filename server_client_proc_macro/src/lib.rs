extern crate proc_macro;

use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn;

use convert_case::{Case, Casing};

mod server_impl;

#[proc_macro]
pub fn server_client(input: TokenStream) -> TokenStream {
    // TODO Add ordered possibility
    let s = syn::parse_macro_input!(input as server_impl::ServerImpl);
    let server_name = format_ident!("{}Server", s.name);
    let mut mod_name = format_ident!("{}Mod", s.name);
    mod_name = syn::Ident::new(&mod_name.to_string().to_case(Case::Snake), mod_name.span());
    let client_name = format_ident!("{}Client", s.name);
    let fields = s.values;
    let fields_names: Vec<syn::Ident> = fields.iter().map(|x| x.name.clone()).collect();
    let methods_blocks: Vec<syn::Block> = s.methods.iter().map(|(_, b)| b.clone()).collect();
    let methods_names: Vec<syn::Ident> = s.methods.iter().map(|(s, _)| s.ident.clone()).collect();
    let methods_names_uppercase: Vec<syn::Ident> = s
        .methods
        .iter()
        .map(|(s, _)| {
            syn::Ident::new(
                &s.ident.to_string().to_case(Case::UpperCamel),
                s.ident.span(),
            )
        })
        .collect();
    let methods_types: Vec<Vec<syn::Type>> = s
        .methods
        .iter()
        .map(|(s, _)| {
            s.inputs
                .iter()
                .map(|x| {
                    if let syn::FnArg::Typed(p) = x {
                        *p.ty.clone()
                    } else {
                        unreachable!("No self parameter should be used"); //TODO Match this in the parser
                    }
                })
                .collect()
        })
        .collect();
    let methods_argnames: Vec<Vec<syn::Pat>> = s
        .methods
        .iter()
        .map(|(s, _)| {
            s.inputs
                .iter()
                .map(|x| {
                    if let syn::FnArg::Typed(p) = x {
                        *p.pat.clone()
                    } else {
                        unreachable!("No self parameter should be used"); //TODO Match this in the parser
                    }
                })
                .collect()
        })
        .collect();
    let methods_return_types: Vec<proc_macro2::TokenStream> = s
        .methods
        .iter()
        .map(|(s, _)| match &s.output {
            syn::ReturnType::Default => quote! {()},
            syn::ReturnType::Type(_, t) => quote! {#t},
        })
        .collect();
    let generics = s.generics.unwrap_or_default();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let public = s.public.map(|x| quote! {#x}).unwrap_or(quote! {});
    let (server, client) = if s.ordered.is_some() {
        (
            quote! {
                pub struct #server_name #generics {
                    connections: ::server_client::IdMap<::server_client::channel::Sender<Reply>>,
                    receiver: ::server_client::channel::Receiver<(Request, usize)>,
                    sender: ::server_client::channel::Sender<(Request, usize)>,
                    #(#fields),*
                }

                impl#impl_generics #server_name #type_generics #where_clause {
                    pub fn new(#(#fields),*) -> Self {
                        let (sender, receiver) = ::server_client::channel::unbounded();
                        Self {
                            connections: ::server_client::IdMap::new(),
                            receiver,
                            sender,
                            #(#fields_names),*
                        }
                    }

                    pub fn connection(&mut self) -> #client_name {
                        // println!("Connection #{}", self.connections.len());
                        let (tx, rx) = ::server_client::channel::unbounded();
                        let id = self.connections.add(tx);
                        #client_name::new(rx, self.sender.clone(), id)
                    }

                    pub fn tick(&mut self) {
                        while let Ok((req, id)) = self.receiver.try_recv() {
                            let (res, remove) = self.handle(req);
                            if let Some(r) = res {
                                self.connections[id].send(r).unwrap();
                            }
                            if remove {
                                // println!("Dropped connection #{} of {}", i, self.connections.len()-1);
                                self.connections.remove(id);
                                break
                            }
                        }
                    }

                    fn handle(&mut self, req: Request) -> (Option<Reply>, bool) {
                        match req {
                            Request::NewConnection => (Some(Reply::NewConnection(self.connection())), false),

                            Request::DropConnection => (None, true),

                            #(
                                Request::#methods_names_uppercase(#(#methods_argnames),*) => {
                                    (Some(Reply::#methods_names_uppercase(#methods_blocks)), false)
                                }
                            )*
                        }
                    }
                }
            },
            quote! {
                pub struct #client_name {
                    receiver: ::server_client::channel::Receiver<Reply>,
                    sender: ::server_client::channel::Sender<(Request, usize)>,
                    id: usize,
                }

                impl #client_name {
                    fn new(receiver: ::server_client::channel::Receiver<Reply>,
                        sender: ::server_client::channel::Sender<(Request, usize)>,
                        id: usize) -> Self {
                        Self {receiver, sender, id}
                    }

                    #(
                        pub fn #methods_names(&self, #(#methods_argnames: #methods_types),*) -> #methods_return_types {
                            self.sender.send((Request::#methods_names_uppercase(#(#methods_argnames),*), self.id)).unwrap();
                            if let Reply::#methods_names_uppercase(x) = self.receiver.recv().unwrap() {
                                x
                            }else{
                                unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                            }
                        }
                    )*
                }

                impl Clone for #client_name {
                    fn clone(&self) -> Self {
                        self.sender.send((Request::NewConnection, self.id)).unwrap();
                        if let Reply::NewConnection(x) = self.receiver.recv().unwrap() {
                            x
                        }else{
                            unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                        }
                    }
                }

                impl Drop for #client_name {
                    fn drop(&mut self) {
                        self.sender.send((Request::DropConnection, self.id)).unwrap();
                    }
                }
            },
        )
    } else {
        (
            quote! {
                pub struct #server_name #generics {
                    connections: Vec<server_client::DuplexEnd<Reply, Request>>,
                    #(#fields),*
                }

                impl#impl_generics #server_name #type_generics #where_clause {
                    pub fn new(#(#fields),*) -> Self {
                        Self {
                            connections: Vec::new(),
                            #(#fields_names),*
                        }
                    }

                    pub fn connection(&mut self) -> #client_name {
                        println!("Connection #{}", self.connections.len());
                        let (x1, x2) = server_client::duplex();
                        self.connections.push(x1);
                        #client_name::new(x2)
                    }

                    pub fn tick(&mut self) {
                        for i in 0..self.connections.len() {
                            if let Ok(m) = self.connections[i].try_recv() {
                                let (res, remove) = self.handle(m);
                                if let Some(r) = res {
                                    self.connections[i].send(r).unwrap();
                                }
                                if remove {

                                println!("Dropped connection #{} of {}", i, self.connections.len()-1);
                                    self.connections.remove(i);
                                    break
                                }
                            }
                        }
                    }

                    fn handle(&mut self, req: Request) -> (Option<Reply>, bool) {
                        match req {
                            Request::NewConnection => (Some(Reply::NewConnection(self.connection())), false),

                            Request::DropConnection => (None, true),

                            #(
                                Request::#methods_names_uppercase(#(#methods_argnames),*) => {
                                    (Some(Reply::#methods_names_uppercase(#methods_blocks)), false)
                                }
                            )*
                        }
                    }
                }
            },
            quote! {
                pub struct #client_name {
                    connection: server_client::DuplexEnd<Request, Reply>
                }

                impl #client_name {
                    fn new(connection: server_client::DuplexEnd<Request, Reply>) -> Self {
                        Self {connection}
                    }

                    #(
                        pub fn #methods_names(&self, #(#methods_argnames: #methods_types),*) -> #methods_return_types {
                            self.connection.send(Request::#methods_names_uppercase(#(#methods_argnames),*)).unwrap();
                            if let Reply::#methods_names_uppercase(x) = self.connection.recv().unwrap() {
                                x
                            }else{
                                unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                            }
                        }
                    )*
                }

                impl Clone for #client_name {
                    fn clone(&self) -> Self {
                        self.connection.send(Request::NewConnection).unwrap();
                        if let Reply::NewConnection(x) = self.connection.recv().unwrap() {
                            x
                        }else{
                            unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                        }
                    }
                }

                impl Drop for #client_name {
                    fn drop(&mut self) {
                        self.connection.send(Request::DropConnection).unwrap();

                    }
                }
            },
        )
    };
    let r = quote! {
        mod #mod_name {
            use super::*;

            enum Request {
                NewConnection,
                DropConnection,
                #(#methods_names_uppercase(#(#methods_types),*)),*
            }

            enum Reply {
                NewConnection(#client_name),
                #(#methods_names_uppercase(#methods_return_types)),*
            }

            #server

            #client
        }
        #public use #mod_name::{#server_name, #client_name};
    };

    r.into()
}
