use std::{collections::HashMap, str::FromStr};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, GenericArgument, Ident, ItemImpl, LitStr, Pat, ReturnType, Token, Type, TypeTuple};

struct MatchGenericT {
    pub ident: syn::Ident,
    pub tp: syn::Type,
    pub is_string: bool,
    pub is_optional: bool,
}
fn match_generic_t(gtype: &str, arg: &FnArg) -> Option<MatchGenericT> {
    let FnArg::Typed(arg) = arg.clone() else {
        return None;
    };
    let Pat::Ident(ident) = *arg.pat else {
        return None;
    };
    let ident = ident.ident.clone();
    // get path last segment
    let p_last_seg = if let syn::Type::Path(p) = *arg.ty {
        p.path.segments.last().cloned()?
    } else {
        return None;
    };
    // is last segment Query ?
    if p_last_seg.ident != gtype {
        return None;
    }
    // get Query<T> or Query<Option<T>> T
    let syn::PathArguments::AngleBracketed(a) = p_last_seg.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(t)) = a.args.into_iter().next() else {
        return None;
    };
    let syn::Type::Path(p) = &t else {
        return None;
    };

    let p_last_seg = p.path.segments.last()?;
    let is_string;
    let is_optional;
    let mut tp = t.clone();
    // is last segment Option ?
    if p_last_seg.ident == "Option" {
        is_optional = true;
        let syn::PathArguments::AngleBracketed(a) = &p_last_seg.arguments else {
            return None;
        };
        let Some(GenericArgument::Type(t)) = a.args.iter().next() else {
            return None;
        };
        let syn::Type::Path(p) = t else {
            return None;
        };
        let p_last_seg = p.path.segments.last()?;
        is_string = p_last_seg.ident == "String";
        tp = t.clone();
    } else if p_last_seg.ident == "String" {
        is_string = true;
        is_optional = false;
    } else {
        is_string = false;
        is_optional = false;
    }
    Some(MatchGenericT {
        ident,
        tp,
        is_string,
        is_optional,
    })
}

fn match_result_t(gtype: &str, ty: &syn::Type) -> Option<syn::Type> {
    // get path last segment
    let p_last_seg = if let syn::Type::Path(p) = ty {
        p.path.segments.last().cloned()?
    } else {
        return None;
    };
    // is last segment Query ?
    if p_last_seg.ident != gtype {
        return None;
    }
    // get Query<T> or Query<Option<T>> T
    let syn::PathArguments::AngleBracketed(ref a) = p_last_seg.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(t)) = a.args.iter().next() else {
        return None;
    };
    let tp = t.clone();
    Some(tp)
}
enum PathItem {
    Literal(String),
    Variant { ident: syn::Ident, tp: syn::Type, is_string: bool },
}
enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for Method {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            _ => Err(syn::Error::new_spanned(s, "expect `get`, `post`, `put` or `delete`")),
        }
    }
}

struct ApiInfo {
    pub name: Ident,
    pub path: Vec<PathItem>,
    pub query: Vec<(String, MatchGenericT)>,
    pub body: syn::Type,
    pub resp: syn::Type,
    pub method: Method,
}

struct ApiInfoBuilder {
    pub name: Ident,
    pub path: Vec<PathItem>,
    pub body: Option<syn::Type>,
    pub resp: syn::Type,
    pub query: Vec<(String, MatchGenericT)>,
    pub method: Option<Method>,
}

impl ApiInfoBuilder {
    pub fn new(name: Ident) -> Self {
        Self {
            name,
            path: Vec::new(),
            body: None,
            resp: syn::Type::Tuple(TypeTuple {
                paren_token: Default::default(),
                elems: Default::default(),
            }),
            method: None,
            query: Vec::new(),
        }
    }
    pub fn build(self) -> Result<ApiInfo, syn::Error> {
        let body = self.body.unwrap_or(syn::Type::Tuple(TypeTuple {
            paren_token: Default::default(),
            elems: Default::default(),
        }));
        let method = self.method.ok_or_else(|| syn::Error::new_spanned(&self.name, "missing method"))?;

        Ok(ApiInfo {
            name: self.name,
            path: self.path,
            body,
            resp: self.resp,
            method,
            query: self.query,
        })
    }
}

/// # Usage
/// This Attribute Macro is used to generate corresponding client methods for you api.
/// Simplely add it **upon** `OpenApi` attribute.
///
/// The `Client` is your custom client struct witch implemented `SimpleInvokeClient` trait.
/// ```no_run, ignore
/// #[simple_invoke_client(Client)]
/// #[poem_openapi::OpenApi(prefix_path = "/ct/msg")]
/// impl Api {
///     #[oai(method = "get", path = "/page")]
///     pub async fn get_page(
///         &self,
///         page_number: Path<u32>,
///         page_size: Query<Option<u32>>,
///         TardisContextExtractor(ctx): TardisContextExtractor,
///     ) -> TardisApiResult<TardisPage<String>> {
///         // do something
///         TardisResp::ok(TardisPage {
///             page_number: 1,
///             page_size: 10,
///             total_size: 1,
///             records: vec!["hello".to_string()],
///         })
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn simple_invoke_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let mut metadata = parse_macro_input!(attr as Metadata);
    // extract openapi metadata
    input.attrs.iter().for_each(|attr| {
        if attr.path().segments.iter().last().is_some_and(|last| last.ident == "OpenApi") {
            let _ = attr.parse_nested_meta(|meta| {
                if metadata.prefix_path.is_none() && meta.path.is_ident("prefix_path") {
                    let path = meta.value()?.parse::<LitStr>()?;
                    metadata.prefix_path.replace(path);
                }
                Ok(())
            });
        }
    });
    let method_info_list = input
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(func) = item {
                let name = &func.sig.ident;

                let mut builder = ApiInfoBuilder::new(name.clone());
                let mut path_map = HashMap::new();
                // 1. find out body: arg with type: Json<T>,
                // 2. find out resp: ReturnType wrapped in TardisApiResult<T>,
                // 3. find out path args: arg with type: Path<T>,
                // 4. find out query args: arg with type: Query<T>,
                for arg in &func.sig.inputs {
                    if let Some(q) = match_generic_t("Query", arg) {
                        builder.query.push((q.ident.to_string(), q));
                    }
                    if let Some(p) = match_generic_t("Path", arg) {
                        path_map.insert(p.ident.to_string(), p);
                    }
                    if let Some(j) = match_generic_t("Json", arg) {
                        builder.body = Some(j.tp);
                    }
                }
                let _oai_metadata = func.attrs.iter().find(|attr| attr.path().is_ident("oai")).map(|attr| {
                    attr.parse_nested_meta(|nested| {
                        if nested.path.is_ident("method") {
                            let method = nested.value()?;
                            let method: LitStr = method.parse()?;
                            let method = Method::from_str(&method.value())?;
                            builder.method = Some(method);
                        }
                        if nested.path.is_ident("path") {
                            let path = nested.value()?.parse::<LitStr>()?.value();
                            builder.path = path
                                .split('/')
                                .filter(|x| !x.is_empty())
                                .map(|x| {
                                    if let Some(ident) = x.strip_prefix(':') {
                                        path_map
                                            .remove(ident)
                                            .map(|arg| PathItem::Variant {
                                                ident: arg.ident,
                                                tp: arg.tp,
                                                is_string: arg.is_string,
                                            })
                                            .unwrap_or(PathItem::Literal(x.to_string()))
                                    } else {
                                        PathItem::Literal(x.to_string())
                                    }
                                })
                                .collect::<Vec<_>>();
                        }
                        Ok(())
                    })
                });
                builder.resp = match &func.sig.output {
                    ReturnType::Type(_, tp) => match_result_t("TardisApiResult", tp).ok_or(syn::Error::new_spanned(&func.sig.output, "expect `TardisApiResult<T>`")).unwrap(),
                    _ => syn::Type::Tuple(TypeTuple {
                        paren_token: Default::default(),
                        elems: Default::default(),
                    }),
                };
                Some(builder.build().unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let client = metadata.client;
    let impl_apis = generate_impl_tardis_api_client(&method_info_list, client, metadata.prefix_path);

    let output = quote! {
        #input
        #impl_apis
    };

    output.into()
}

fn generate_impl_tardis_api_client(apis: &[ApiInfo], client: Type, prefix: Option<LitStr>) -> proc_macro2::TokenStream {
    let mut impl_items = Vec::new();

    for api_info in apis {
        let name = &api_info.name;
        let path = generate_path_tokens(&api_info.path);
        let query = generate_query_tokens(&api_info.query);
        let body = generate_type_tokens(&api_info.body);
        let resp = generate_type_tokens(&api_info.resp);
        let method = generate_method_token(&api_info.method);
        let body_resp = match &api_info.method {
            Method::Get | Method::Delete => quote!( #resp ),
            Method::Post | Method::Put => quote!( #body => #resp ),
        };
        let path = match &prefix {
            Some(prefix) => quote! { #prefix, #path },
            None => quote! { #path },
        };
        let item = quote! {
            { #name, #method [#path] {#query} #body_resp }
        };

        impl_items.push(item);
    }

    quote! {
        bios_sdk_invoke::impl_tardis_api_client! {
            #client:
            #(#impl_items)*
        }
    }
}

fn generate_path_tokens(path: &[PathItem]) -> proc_macro2::TokenStream {
    let tokens = path.iter().map(|item| match item {
        PathItem::Literal(s) => quote! { #s },
        PathItem::Variant { ident, tp, is_string } => {
            if *is_string {
                quote! { #ident }
            } else {
                quote! { #ident: #tp }
            }
        }
    });

    quote! { #(#tokens),* }
}

fn generate_query_tokens(query: &[(String, MatchGenericT)]) -> proc_macro2::TokenStream {
    let tokens = query.iter().map(|(_name, ty)| {
        let ty_ts = generate_type_tokens(&ty.tp);
        let ident = &ty.ident;
        match (ty.is_optional, ty.is_string) {
            (true, true) => quote! { #ident? },
            (true, false) => quote! { #ident?: #ty_ts },
            (false, true) => quote! { #ident },
            (false, false) => quote! { #ident: #ty_ts },
        }
    });

    quote! { #(#tokens),* }
}

fn generate_type_tokens(ty: &Type) -> proc_macro2::TokenStream {
    quote! { #ty }
}

fn generate_method_token(method: &Method) -> proc_macro2::TokenStream {
    match method {
        Method::Get => quote! { get },
        Method::Post => quote! { post },
        Method::Put => quote! { put },
        Method::Delete => quote! { delete },
    }
}

struct Metadata {
    client: syn::Type,
    prefix_path: Option<syn::LitStr>,
}

impl syn::parse::Parse for Metadata {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let client = input.parse::<Type>()?;
        let mut meta_data = Self { client, prefix_path: None };
        if let Ok(_comma) = input.parse::<Token![,]>() {
            let prefix_path = Some(input.parse::<LitStr>()?);
            meta_data.prefix_path = prefix_path;
        }
        Ok(meta_data)
    }
}
