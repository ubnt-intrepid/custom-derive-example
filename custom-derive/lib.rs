extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod subcommand;
mod util;

use proc_macro::TokenStream;
use syn::Body;
use subcommand::Subcommand;

#[proc_macro_derive(App, attributes(clap))]
pub fn derive_clap_app(input: TokenStream) -> TokenStream {
  let ast = syn::parse_derive_input(&input.to_string()).unwrap();
  let quotes = match ast.body {
    Body::Enum(variants) => {
      let subcommand = Subcommand::new(ast.ident, ast.attrs, variants).unwrap();
      quote!(#subcommand)
    }
    Body::Struct(_) => panic!("#[derive(Subcommand)] is only supported for enum"),
  };

  quotes.parse().unwrap()
}
