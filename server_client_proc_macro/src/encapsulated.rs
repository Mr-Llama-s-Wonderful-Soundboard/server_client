use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, ImplItem, ImplItemMethod, Result, Token, Type};

pub struct Attrs {
    pub public: Option<Token![pub]>,
    pub asyncness: Option<Token![async]>,
    pub ordered: Option<super::kw::ordered>,
    pub name: Option<Ident>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let public = if input.lookahead1().peek(Token![pub]) {
            Some(input.parse::<Token![pub]>()?)
        } else {
            None
        };
        let asyncness = if input.lookahead1().peek(Token![async]) {
            let parsed = input.parse::<Token![async]>()?;
            if cfg!(not(feature = "async")) {
                return Err(Error::new(
                    parsed.span,
                    "Can't use async modifier as the async feature for server_client is disabled",
                ));
            }
            Some(parsed)
        } else {
            None
        };
        let ordered = if input.lookahead1().peek(super::kw::ordered) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = if input.lookahead1().peek(Ident) {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(Self {
            public,
            asyncness,
            ordered,
            name,
        })
    }
}

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct Method {
    pub t: TokenStream,
}

impl Method {
    fn try_from_method(value: &ImplItemMethod, is_async: bool, ty: &Type) -> Result<Self> {
        let await_call = if let Some(a) = value.sig.asyncness {
            if !is_async {
                return Err(Error::new_spanned(
                    a,
                    "Can't encapsulate async method in non async server",
                ));
            }
            Some(quote! {.await})
        } else {
            None
        };
        if let (Some(lt), Some(gt)) = &(value.sig.generics.lt_token, value.sig.generics.gt_token) {
			let mut tokens = lt.to_token_stream();
			tokens.extend(vec![gt.to_token_stream()]);
            return Err(Error::new_spanned(
                tokens,
                "Can't encapsulate method with generics",
            ));
        }
        if let Some(where_clause) = &value.sig.generics.where_clause {
            return Err(Error::new_spanned(
                where_clause,
                "Can't encapsulate method with where clause",
            ));
        }
        let mut is_method = false;
        for x in value.sig.inputs.pairs() {
            if let syn::FnArg::Receiver(_) = x.value() {
                is_method = true;
                break;
            }
        }
        let args: syn::punctuated::Punctuated<syn::FnArg, Token![,]> = value
            .sig
            .inputs
            .clone()
            .into_pairs()
            .filter(|x| {
                if let syn::FnArg::Typed(_) = x.value() {
                    true
                } else {
                    false
                }
            })
			.collect();
		let args_names: Result<syn::punctuated::Punctuated<Box<syn::Pat>, Token![,]>> = value
		.sig
		.inputs
		.clone()
		.into_pairs()
		.filter(|x| {
			if let syn::FnArg::Typed(_) = x.value() {
				true
			} else {
				false
			}
		})
		.map(|x|{
			let (v, p) = x.into_tuple();
			if let syn::FnArg::Typed(t) = v {
				Ok(syn::punctuated::Pair::new(t.pat, p))
			}else{
				Err(Error::new_spanned(v, "Unexpected self"))
			}
		})
		.collect();
        let args_names= args_names?;
        // println!("{:?}", args_names);
        let fn_tok = value.sig.fn_token;
        let name = value.sig.ident.clone();
        if name == "new" {
            return Err(Error::new_spanned(
                name,
                "Can't encapsulate method with name new",
            ));
        }
		let output = value.sig.output.clone();
		let call = if is_method {quote! {self.value.#name(#args_names)#await_call}} else {quote! {#ty::#name(#args_names)#await_call}};
		// println!("Call: {}", call);
        Ok(Self {
            t: quote! {
                #fn_tok #name(#args) #output {
                    #call
                }
            },
        })
    }

    pub fn try_from_item(value: &ImplItem, is_async: bool, ty: &Type) -> Result<Self> {
        match value {
            ImplItem::Method(m) => Self::try_from_method(m, is_async, ty),
            x => Err(Error::new_spanned(
                x,
                "Can't encapsulate impls with non methods (use another impl block)",
            )),
        }
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.t.to_tokens(tokens)
    }
}
