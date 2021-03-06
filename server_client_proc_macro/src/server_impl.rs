use syn::parse::{Parse, ParseStream};
// use syn::punctuated::Punctuated;
// use syn::spanned::Spanned;
use syn::{braced, Block, Generics, Ident, ImplItemMethod, Result, Signature, Token, Type, Error};

// use proc_macro2::Span;
use quote::ToTokens;

pub struct ServerImpl {
    pub name: Ident,
    pub values: Vec<Field>,
    pub methods: Vec<(Signature, Block)>,
    pub generics: Option<Generics>,
    pub public: Option<Token![pub]>,
    pub ordered: Option<super::kw::ordered>,
    pub asynchronous: Option<Token![async]>
}

impl Parse for ServerImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let public = if input.lookahead1().peek(Token![pub]) {
            Some(input.parse::<Token![pub]>()?)
        } else {
            None
        };
        let asynchronous = if input.lookahead1().peek(Token![async]) {
            let parsed = input.parse::<Token![async]>()?;
            if cfg!(not(feature = "async")) {
                return Err(Error::new(parsed.span, "Can't use async modifier as the async feature for server_client is disabled"))
            }
            Some(parsed)
        } else {
            None
        };
        let ordered = if input.lookahead1().peek(super::kw::ordered) {
            Some(input.parse()?)
        }else{
            None
        };
        let name = input.parse()?;
        let generics = if input.lookahead1().peek(Token![<]) {
            Some(Generics::parse(input)?)
        } else {
            None
        };
        let content;
        braced!(content in input);
        let mut methods = Vec::new();
        let mut values = Vec::new();
        while !content.is_empty() {
            if content.lookahead1().peek(Token![let]) {
                let field = content.parse()?;
                values.push(field);
            } else if content.lookahead1().peek(Token![fn]) {
                let method: ImplItemMethod = content.parse()?;
                methods.push((method.sig, method.block));
            } else {
                return Err(content.lookahead1().error());
            }
        }
        Ok(Self {
            name,
            values,
            methods,
            generics,
            public,
            ordered,
            asynchronous
        })
    }
}

impl std::fmt::Debug for ServerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:?} {{\n{}}}",
            self.name.to_token_stream(),
            self.values,
            self.methods
                .iter()
                .fold(String::new(), |s, (sig, block)| format!(
                    "{}{} {}\n",
                    s,
                    sig.to_token_stream(),
                    block.to_token_stream()
                ))
        )
    }
}

pub struct Field {
    pub name: Ident,
    typ: Type,
    colon: Token![:],
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let _let: Token![let] = input.parse()?;
        let name: Ident = input.parse()?;
        if input.lookahead1().peek(Token![:]) {
            let colon = input.parse()?;
            let typ: Type = input.parse()?;
            Ok(Self { name, typ, colon })
        } else {
            Err(input.lookahead1().error())
        }
    }
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name.to_token_stream(),
            self.typ.to_token_stream()
        )
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.typ.to_tokens(tokens);
    }
}
