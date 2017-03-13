extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod app;
mod attr;
mod subcommand;
mod util;

use proc_macro::TokenStream;
use syn::{Body, VariantData};
use quote::{Tokens};

use app::App;
use subcommand::Subcommand;


#[proc_macro_derive(App, attributes(clap))]
pub fn derive_clap_app(input: TokenStream) -> TokenStream {
  let s = input.to_string();
  let ast = syn::parse_derive_input(&s).unwrap();
  let parsed = Parsed::new(ast).unwrap();

  let mut tokens = Tokens::new();
  parsed.derive_app(&mut tokens);
  parsed.derive_from_arg_matches(&mut tokens);

  tokens.parse().unwrap()
}

trait DeriveApp {
  fn derive_app(&self, &mut quote::Tokens);
}

trait DeriveFromArgMatches {
  fn derive_from_arg_matches(&self, &mut quote::Tokens);
}


enum Parsed {
  App(App),
  Subcommand(Subcommand),
}

impl Parsed {
  fn new(ast: syn::DeriveInput) -> Result<Parsed, String> {
    match ast.body {
      Body::Enum(variants) => {
        let subcommand = Subcommand::new(ast.ident, ast.attrs, variants)?;
        Ok(Parsed::Subcommand(subcommand))
      }
      Body::Struct(data) => {
        let app = match data {
          VariantData::Unit => App::new(ast.ident, ast.attrs, Vec::new())?,
          VariantData::Struct(fields) => App::new(ast.ident, ast.attrs, fields)?,
          VariantData::Tuple(_) => {
            return Err("#[derive(App)] is not supported for tuple struct".into())
          }
        };
        Ok(Parsed::App(app))
      }
    }
  }
}

impl DeriveApp for Parsed {
  fn derive_app(&self, tokens: &mut quote::Tokens) {
    match *self {
      Parsed::App(ref app) => app.derive_app(tokens),
      Parsed::Subcommand(ref subcommand) => subcommand.derive_app(tokens),
    }
  }
}

impl DeriveFromArgMatches for Parsed {
  fn derive_from_arg_matches(&self, tokens: &mut quote::Tokens) {
    match *self {
      Parsed::App(ref app) => app.derive_from_arg_matches(tokens),
      Parsed::Subcommand(ref subcommand) => subcommand.derive_from_arg_matches(tokens),
    }
  }
}