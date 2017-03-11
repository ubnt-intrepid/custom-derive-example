extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{MetaItem, NestedMetaItem, Lit};

#[proc_macro_derive(Subcommand, attributes(clap))]
pub fn subcommand(input: TokenStream) -> TokenStream {
  let subcommand = syn::parse_derive_input(&input.to_string())
    .and_then(Subcommand::new)
    .unwrap();

  subcommand.to_derived_tokens()
    .parse()
    .unwrap()
}

struct SubcommandAttribute {
  name: quote::Tokens,
  about: quote::Tokens,
}

impl SubcommandAttribute {
  fn new(attrs: Vec<syn::Attribute>) -> Result<SubcommandAttribute, String> {
    let mut name = None;
    let mut about = None;

    for attr in attrs.into_iter().filter(|attr| attr.name() == "clap") {
      let items = match attr.value {
        MetaItem::List(_, ref items) => items,
        _ => continue,
      };

      for item in items {
        let (ident, value) = match *item { 
          NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, Lit::Str(ref value, _))) => {
            (ident, value)
          }
          _ => return Err("invalud attribute".into()),
        };

        match ident.as_ref() {
          "name" => name = Some(value.to_owned()),
          "about" => about = Some(value.to_owned()),
          _ => return Err("invalid attribute name".into()),
        }
      }
    }

    let name = name.map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_NAME")));
    let about = about.map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_DESCRIPTION")));

    Ok(SubcommandAttribute {
      name: name,
      about: about,
    })
  }
}

struct Subcommand {
  ident: syn::Ident,
  attr: SubcommandAttribute,
  variants: Vec<syn::Variant>,
}

impl Subcommand {
  fn new(ast: syn::DeriveInput) -> Result<Subcommand, String> {
    let ident = ast.ident;
    let attr = SubcommandAttribute::new(ast.attrs)?;

    match ast.body {
      syn::Body::Enum(variants) => {
        Ok(Subcommand {
          ident: ident,
          attr: attr,
          variants: variants,
        })
      }
      syn::Body::Struct(_) => Err("#[derive(Subcommand)] is only supported for enum".into()),
    }
  }

  fn to_derived_tokens(&self) -> quote::Tokens {
    let ident = &self.ident;
    let name = &self.attr.name;
    let about = &self.attr.about;

    let subcommand_apps = {
      join_tokens(self.variants.iter()
        .map(|v| {
          let (name, help) = get_attr_name_and_help(&v);
          let help = help.unwrap_or("Put help message".to_owned());
          quote! {
        .subcommand(clap::SubCommand::with_name(#name).about(#help))
      }
        }))
    };

    let subcommand_froms = {
      join_tokens(self.variants.iter()
        .map(|v| {
          let (name, _) = get_attr_name_and_help(&v);
          let variant = &v.ident;
          quote! { (#name, _) => #ident :: #variant(Default::default()), }
        }))
    };

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

fn join_tokens<I, T>(iter: I) -> quote::Tokens
  where I: IntoIterator<Item = T>,
        T: quote::ToTokens
{
  let mut tokens = quote::Tokens::new();
  tokens.append_all(iter);
  tokens
}
