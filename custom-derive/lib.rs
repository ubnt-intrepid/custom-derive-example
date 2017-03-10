extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Subcommand)]
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

fn impl_subcommand_app(variants: &[syn::Variant]) -> quote::Tokens {
  variants.into_iter()
    .map(|v| {
      let name = v.ident.as_ref().to_lowercase();
      quote! { .subcommand(clap::SubCommand::with_name(#name)) }
    })
    .fold(quote::Tokens::new(), |mut acc, t| {
      acc.append(t.as_ref());
      acc
    })
}

fn impl_subcommand_from(enumname: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
  let mut tokens = quote::Tokens::new();
  let variants = variants.into_iter()
    .map(|v| {
      let ident = &v.ident;
      let name = v.ident.as_ref().to_lowercase();
      quote! { (#name, _) => #enumname :: #ident(Default::default()) }
    });
  tokens.append_separated(variants, ",");
  tokens.append(",");
  tokens
}
