extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote, ToTokens};
use syn;

use convert_case::{Case, Casing};

mod server_impl;

#[proc_macro]
pub fn server_client(input: TokenStream) -> TokenStream {
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
    let await_call = if s.asynchronous.is_some() { quote! {.await}} else {quote!{}};
    let async_token = if let Some(t) = s.asynchronous {quote! {#t}} else {quote! {}};
    let (server, client) = if s.ordered.is_some() {
        let (sender, receiver, create) = if s.asynchronous.is_some() {
            (quote!{::server_client::asynchronous::Sender}, quote!{::server_client::asynchronous::Receiver}, quote!{::server_client::asynchronous::unbounded})
        }else{
            (quote!{::server_client::channel::Sender}, quote!{::server_client::channel::Receiver}, quote!{::server_client::channel::unbounded})
        };
        let (clone_impl, clone_method) = if s.asynchronous.is_some() {
            (quote! {}, quote!{
                pub #async_token fn clone(&mut self) -> Self {
                    self.sender.send((Request::NewConnection, self.id)).unwrap();
                    if let Reply::NewConnection(x) = self.receiver.recv().await.unwrap() {
                        x
                    }else{
                        unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                    }
                }
            })
        } else {
            (quote! {
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
            }, quote! {})
        };
        (
            quote! {
                pub struct #server_name #generics {
                    connections: ::server_client::IdMap<#sender<Reply>>,
                    receiver: #receiver<(Request, usize)>,
                    sender: #sender<(Request, usize)>,
                    #(#fields),*
                }

                impl#impl_generics #server_name #type_generics #where_clause {
                    pub fn new(#(#fields),*) -> Self {
                        let (sender, receiver) = #create();
                        Self {
                            connections: ::server_client::IdMap::new(),
                            receiver,
                            sender,
                            #(#fields_names),*
                        }
                    }

                    pub fn connection(&mut self) -> #client_name {
                        // println!("Connection #{}", self.connections.len());
                        let (tx, rx) = #create();
                        let id = self.connections.add(tx);
                        #client_name::new(rx, self.sender.clone(), id)
                    }

                    pub #async_token fn tick(&mut self) {
                        while let Ok((req, id)) = self.receiver.try_recv()#await_call {
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
                    receiver: #receiver<Reply>,
                    sender: #sender<(Request, usize)>,
                    id: usize,
                }

                impl #client_name {
                    fn new(receiver: #receiver<Reply>,
                        sender: #sender<(Request, usize)>,
                        id: usize) -> Self {
                        Self {receiver, sender, id}
                    }

                    #(
                        pub #async_token fn #methods_names(&mut self, #(#methods_argnames: #methods_types),*) -> #methods_return_types {
                            self.sender.send((Request::#methods_names_uppercase(#(#methods_argnames),*), self.id)).unwrap();
                            if let Reply::#methods_names_uppercase(x) = self.receiver.recv()#await_call.unwrap() {
                                x
                            }else{
                                unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                            }
                        }
                    )*

                    #clone_method
                }

                #clone_impl

                impl std::fmt::Debug for #client_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "#client_name[{}]", self.id)
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
        let (duplex, duplex_create) = if s.asynchronous.is_some() {
            (quote!{::server_client::asynchronous::DuplexEnd}, quote!{::server_client::asynchronous::duplex})
        }else{
            (quote!{::server_client::DuplexEnd}, quote!{::server_client::duplex})
        };
        let (clone_impl, clone_method) = if s.asynchronous.is_some() {
            (quote! {}, quote!{
                pub #async_token fn clone(&mut self) -> Self {
                    self.connection.send(Request::NewConnection).unwrap();
                    if let Reply::NewConnection(x) = self.connection.recv().await.unwrap() {
                        x
                    }else{
                        unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                    }
                }
            })
        } else {
            (quote! {
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
            }, quote! {})
        };
        (
            quote! {
                pub struct #server_name #generics {
                    connections: Vec<#duplex<Reply, Request>>,
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
                        let (x1, x2) = #duplex_create();
                        self.connections.push(x1);
                        #client_name::new(x2)
                    }

                    pub #async_token fn tick(&mut self) {
                        for i in 0..self.connections.len() {
                            if let Ok(m) = self.connections[i].try_recv()#await_call {
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
                    connection: #duplex<Request, Reply>
                }

                impl #client_name {
                    fn new(connection: #duplex<Request, Reply>) -> Self {
                        Self {connection}
                    }

                    #(
                        pub #async_token fn #methods_names(&mut self, #(#methods_argnames: #methods_types),*) -> #methods_return_types {
                            self.connection.send(Request::#methods_names_uppercase(#(#methods_argnames),*)).unwrap();
                            if let Reply::#methods_names_uppercase(x) = self.connection.recv()#await_call.unwrap() {
                                x
                            }else{
                                unreachable!("Unexpected non return for fn (is this client being used in parallel?)");
                            }
                        }
                    )*

                    #clone_method
                }

                #clone_impl

                impl std::fmt::Debug for #client_name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "#client_name")
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
        pub mod #mod_name {
            use super::*;

            #[derive(Debug)]
            enum Request {
                NewConnection,
                DropConnection,
                #(#methods_names_uppercase(#(#methods_types),*)),*
            }

            #[derive(Debug)]
            enum Reply {
                NewConnection(#client_name),
                #(#methods_names_uppercase(#methods_return_types)),*
            }

            #server

            #client
        }
        #[allow(unused_imports)]
        #public use #mod_name::{#server_name, #client_name};
    };

    r.into()
}

mod encapsulated;

#[proc_macro_attribute]
pub fn encapsulate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = syn::parse_macro_input!(item as syn::ItemImpl);
    if let Some((n, path, for_kw)) = impl_block.trait_ {
        let mut tokens = n.map(|x|x.to_token_stream()).unwrap_or(TokenStream2::new());
        tokens.extend(vec![path.to_token_stream(), for_kw.to_token_stream()]);
        return syn::Error::new_spanned(tokens, "Can't encapsulate in a server-client a trait implementation").to_compile_error().into()
    }
    let attrs = syn::parse_macro_input!(attr as encapsulated::Attrs);
    let ty = &impl_block.self_ty;
    let ty_tokens = ty.to_token_stream().into();
    let public = attrs.public.map(|x| quote!{#x}).unwrap_or(quote!{});
    let ordered = attrs.ordered.map(|x| quote !{#x}).unwrap_or(quote!{});
    let asyncness = attrs.asyncness.map(|x| quote!{#x}).unwrap_or(quote!{});
    // let type_name = syn::parse_macro_input!(ty as syn::Ident);
    let name = attrs.name.unwrap_or(syn::parse_macro_input!(ty_tokens as syn::Ident));
    let mut methods = Vec::new();
    for item in &impl_block.items {
        match encapsulated::Method::try_from_item(item, attrs.asyncness.is_some(), ty) {
            Ok(v) => methods.push(v),
            Err(e) => return e.to_compile_error().into()
        }
    }
    let macro_tokens = quote! {
        #public #asyncness #ordered #name {
            let value: #ty

            #(#methods)*
        }
    };
    println!("{}", macro_tokens);
    let macro_res: TokenStream2 = server_client(macro_tokens.into()).into();
    (quote!{
        #impl_block
        #macro_res
    }).into()
}

mod kw {
    syn::custom_keyword!(ordered);
}