extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{MetaItem, NestedMetaItem, Lit};

#[proc_macro_derive(Subcommand, attributes(clap))]
pub fn subcommand(input: TokenStream) -> TokenStream {
  let ast = syn::parse_derive_input(&input.to_string()).unwrap();
  let gen = impl_subcommand(&ast);
  gen.parse().unwrap()
}

fn impl_subcommand(ast: &syn::DeriveInput) -> quote::Tokens {
  let mut name = None;
  let mut about = None;

  for attr in ast.attrs.iter().filter(|attr| attr.name() == "clap") {
    let items = match attr.value {
      MetaItem::List(_, ref items) => items,
      _ => continue,
    };

    for item in items {
      let (ident, value) = match *item { 
        NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, Lit::Str(ref value, _))) => {
          (ident, value)
        }
        _ => unreachable!(),
      };

      match ident.as_ref() {
        "name" => name = Some(value.to_owned()),
        "about" => about = Some(value.to_owned()),
        _ => unreachable!(),
      }
    }
  }

  let name = name.map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_NAME")));
  let about = about.map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_DESCRIPTION")));

  let ref variants = match ast.body {
    syn::Body::Enum(ref variants) => variants,
    syn::Body::Struct(_) => unreachable!(),
  };

  let ident = &ast.ident;
  let subcommand_apps = impl_subcommand_app(variants);
  let subcommand_froms = impl_subcommand_from(ident, variants);

  quote! {
    impl Subcommand for #ident {
      fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
        clap::App::new(#name)
          .about(#about)
          .setting(clap::AppSettings::VersionlessSubcommands)
          .setting(clap::AppSettings::SubcommandRequiredElseHelp)
          #subcommand_apps
      }
    }
    impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #ident {
      fn from(m: &'b clap::ArgMatches<'a>) -> Self {
        match m.subcommand() {
          #subcommand_froms
          _ => unreachable!(),
        }
      }
    }
  }
}

fn get_attr_name_and_help(v: &syn::Variant) -> (String, Option<String>) {
  let mut name = None;
  let mut help = None;

  for attr in v.attrs.iter().filter(|attr| attr.name() == "clap") {
    let items = match attr.value {
      MetaItem::List(_, ref items) => items,
      _ => continue,
    };

    for item in items {
      let (ident, value) = match *item {
        NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, Lit::Str(ref value, _))) => {
          (ident, value)
        }
        _ => unreachable!(),
      };

      match ident.as_ref() {
        "name" => name = Some(value.to_owned()),
        "help" => help = Some(value.to_owned()),
        _ => unreachable!(),
      }
    }
  }

  let name = name.unwrap_or_else(|| v.ident.as_ref().to_lowercase());

  (name, help)
}

fn impl_subcommand_app(variants: &[syn::Variant]) -> quote::Tokens {
  join_tokens(variants.into_iter()
    .map(|v| {
      let (name, help) = get_attr_name_and_help(&v);
      let help = help.unwrap_or("Put help message".to_owned());
      quote! {
        .subcommand(clap::SubCommand::with_name(#name).about(#help))
      }
    }))
}

fn impl_subcommand_from(enumname: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
  join_tokens(variants.into_iter()
    .map(|v| {
      let (name, _) = get_attr_name_and_help(&v);
      let ident = &v.ident;
      quote! { (#name, _) => #enumname :: #ident(Default::default()), }
    }))
}

fn join_tokens<I, T>(iter: I) -> quote::Tokens
  where I: IntoIterator<Item = T>,
        T: quote::ToTokens
{
  let mut tokens = quote::Tokens::new();
  tokens.append_all(iter);
  tokens
}
