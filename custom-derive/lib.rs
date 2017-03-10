extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Subcommand, attributes(clap))]
pub fn subcommand(input: TokenStream) -> TokenStream {
  let ast = syn::parse_derive_input(&input.to_string()).unwrap();
  let gen = impl_subcommand(&ast);
  gen.parse().unwrap()
}

fn impl_subcommand(ast: &syn::DeriveInput) -> quote::Tokens {
  let ref variants = match ast.body {
    syn::Body::Enum(ref variants) => variants,
    syn::Body::Struct(_) => unreachable!(),
  };

  let name = &ast.ident;
  let subcommand_apps = impl_subcommand_app(variants);
  let subcommand_froms = impl_subcommand_from(name, variants);

  quote! {
    impl Subcommand for #name {
      fn app<'a, 'b: 'a>(name: &str) -> clap::App<'a, 'b> {
        clap::App::new(name)
          .setting(clap::AppSettings::VersionlessSubcommands)
          .setting(clap::AppSettings::SubcommandRequiredElseHelp)
          #subcommand_apps
      }
    }
    impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #name {
      fn from(m: &'b clap::ArgMatches<'a>) -> Self {
        match m.subcommand() {
          #subcommand_froms
          _ => panic!(""),
        }
      }
    }
  }
}

fn get_attr_name_and_help(v: &syn::Variant) -> (String, Option<String>) {
  use syn::{MetaItem, NestedMetaItem, Lit};

  let mut name = None;
  let mut help = None;

  for attr in &v.attrs {
    let attr: &syn::Attribute = attr;
    if let MetaItem::List(ref ident, ref items) = attr.value {
      if ident.as_ref() == "clap" {
        for item in items {
          if let NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident,
                                                              Lit::Str(ref value, _))) = *item {
            match ident.as_ref() {
              "name" => name = Some(value.to_owned()),
              "help" => help = Some(value.to_owned()),
              _ => unreachable!(),
            }
          } else {
            unreachable!()
          }
        }
      } else {
        unreachable!()
      }
    } else {
      unreachable!()
    }
  }

  (name.unwrap_or_else(|| v.ident.as_ref().to_lowercase()), help)
}

fn impl_subcommand_app(variants: &[syn::Variant]) -> quote::Tokens {
  let variants = variants.into_iter()
    .map(|v| {
      let (name, help) = get_attr_name_and_help(&v);
      let help = help.unwrap_or("Put help message".to_owned());
      quote! {
        .subcommand(clap::SubCommand::with_name(#name).about(#help))
      }
    });

  let mut tokens = quote::Tokens::new();
  tokens.append_all(variants);
  tokens
}

fn impl_subcommand_from(enumname: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
  let variants = variants.into_iter()
    .map(|v| {
      let (name, _) = get_attr_name_and_help(&v);
      let ident = &v.ident;
      quote! { (#name, _) => #enumname :: #ident(Default::default()), }
    });

  let mut tokens = quote::Tokens::new();
  tokens.append_all(variants);
  tokens
}
