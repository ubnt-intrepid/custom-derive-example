extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod subcommand;
mod util;

use proc_macro::TokenStream;
use subcommand::Subcommand;

#[proc_macro_derive(App, attributes(clap))]
pub fn subcommand(input: TokenStream) -> TokenStream {
  let subcommand = syn::parse_derive_input(&input.to_string())
    .and_then(Subcommand::new)
    .unwrap();

  subcommand.to_derived_tokens()
    .parse()
    .unwrap()
}
