extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
extern crate structopt;
extern crate structopt_derive;

use proc_macro::TokenStream;
use syn::*;
use syn::punctuated::Punctuated;
use syn::token::{Comma};
use structopt::*;
use structopt_derive::*;

#[proc_macro_derive(PlatformConfig, attributes(platformconfig))]
pub fn platformconfig(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    impl_platformconfig(&input).into()
}

fn impl_platformconfig(input: &DeriveInput) -> quote::Tokens {
    use syn::Data::*;

    let struct_name = &input.ident;
    let inner_impl = match input.data {
        Struct(DataStruct { fields: syn::Fields::Named(ref fields), .. }) =>
            impl_platformconfig_for_struct(struct_name, &fields.named),
        _ => panic!("platformconfig only supports non-tuple structs")
    };

    quote!(#inner_impl)
}

use syn::Meta::*;

fn is_option(ty: &Type) -> bool {
    if let syn::Type::Path(TypePath { path: syn::Path { ref segments, .. }, .. }) = *ty {
        segments.iter().last().unwrap().ident.as_ref() == "Option"
    } else {
        false
    }
}

struct FieldInfo<'a> {
    pub field: &'a Field,
    pub structopt_attrs: Vec<Attribute>,
    pub is_optional: bool,
    pub is_optional_cmd: bool,
    pub is_cmd_arg: bool
}

impl<'a> FieldInfo<'a> {
    pub fn new(field: &'a Field) -> Self {
        Self {
            field,
            structopt_attrs: Vec::new(),
            is_optional: false,
            is_optional_cmd: false,
            is_cmd_arg: true
        }
    }

    pub fn has_structopt_attrs(&self) -> bool {
        !self.structopt_attrs.is_empty()
    }

    pub fn structopt_field(&self) -> Option<quote::Tokens> {
        if !self.is_cmd_arg { return None }

        let ident = &self.field.ident;
        let vis = &self.field.vis;
        let colon_token = &self.field.colon_token;
        let original_type = &self.field.ty;

        let ty = if !self.is_optional_cmd || self.is_optional {
            quote! { #original_type }
        } else {
            quote! { Option<#original_type> }
        };


        let attrs = if self.has_structopt_attrs() {
            let attrs = &self.structopt_attrs;
            quote! { #(#attrs)* }
        } else {
            quote! {}
        };


        Some(quote! {
            #attrs
            #vis #ident #colon_token #ty
        })
    }
}

fn analyze_field<'a>(field: &'a Field) -> FieldInfo<'a> {
    let mut result = FieldInfo::new(field);

    result.is_optional = is_option(&field.ty);
    result.is_optional_cmd = result.is_optional;

    for attr in field.attrs.iter() {
        let path = &attr.path;

        if quote!(#path) != quote!(platformconfig) {
            continue; // ignore attributes other than platformconfig
        }

        let meta = attr.interpret_meta();

        if let Some(List(list)) = meta {
            let mut structopt_nesteds = Punctuated::<NestedMeta, Comma>::new();

            for nested in list.nested {
                let is_platformconfig_nested = match nested {
                    NestedMeta::Meta(Word(ident)) if ident == Ident::from("optional_cmd") => {
                        result.is_optional_cmd = true;
                        true
                    },
                    NestedMeta::Meta(Word(ident)) if ident == Ident::from("no_cmd") => {
                        result.is_cmd_arg = false;
                        true
                    },
                    _ => false
                };

                if !is_platformconfig_nested {
                    structopt_nesteds.push(nested);
                }
            }

            if !structopt_nesteds.is_empty() {
                result.structopt_attrs.push(Attribute { path: "structopt".into(), tts: quote!((#structopt_nesteds)).into(), ..attr.clone() });
            }
        }
    }

    result
}

use proc_macro2::Span;
fn impl_platformconfig_trait(name: &Ident, field_infos: &[FieldInfo]) -> quote::Tokens {
    let field_setters = field_infos
        .iter()
        .map(|fi| {
            // TODO: error handling
            let ident = fi.field.ident;
            let ident_str = syn::Lit::Str(LitStr::new(&ident.unwrap().to_string(), Span::call_site()));

            match (fi.is_cmd_arg, fi.is_optional_cmd, fi.is_optional) {
                (true, true, true)   => quote! { #ident: if matches.is_present(#ident_str) { opts.#ident } else { config.get(#ident_str).ok() } },
                (true, true, false)  => quote! { #ident: if matches.is_present(#ident_str) { opts.#ident.unwrap() } else { config.get(#ident_str).unwrap() } },
                (true, false, _)     => quote! { #ident: if matches.is_present(#ident_str) { opts.#ident } else { config.get(#ident_str).unwrap() } },
                (false, _, true)     => quote! { #ident: config.get(#ident_str).ok() },
                (false, _, false)    => quote! { #ident: config.get(#ident_str).unwrap() }
            }
        });

    let structopt_struct_name = Ident::from(format!("PlatformConfig{}StructOpt", name));

    quote!{
        impl From<config::Config> for #name {
            fn from(config: config::Config) -> Self {
                use structopt::StructOpt;

                let matches = #structopt_struct_name::clap().get_matches();
                let opts = #structopt_struct_name::from_clap(&matches);

                Self {
                    #(#field_setters),*
                }
            }
        }
    }
}

fn gen_structopt_struct(name: &Ident, field_infos: &[FieldInfo]) -> quote::Tokens {
    let name = Ident::from(format!("PlatformConfig{}StructOpt", name));
    let fields = field_infos
        .iter()
        .filter_map(|fi| fi.structopt_field())
        .collect::<Vec<quote::Tokens>>();

    quote! {
        #[derive(StructOpt)]
        struct #name {
            #(#fields),*
        }
    }
}

fn impl_platformconfig_for_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>
) -> quote::Tokens {
    let field_infos = fields
        .iter()
        .map(analyze_field)
        .collect::<Vec<_>>();

    let platformconfig_trait_impl = impl_platformconfig_trait(name, &field_infos);
    let structopt = gen_structopt_struct(name, &field_infos);

    quote! {
        #platformconfig_trait_impl
        #structopt
    }
}
