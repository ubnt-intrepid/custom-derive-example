use syn::{Attribute, Field, Ident};
use quote::{Tokens};
use attr;

pub struct App {
  ident: Ident,
  attrs: attr::Attributes,
  fields: Vec<Field>,
}

impl App {
  pub fn new(ident: Ident, attrs: Vec<Attribute>, fields: Vec<Field>) -> Result<App, String> {
    let attrs = attr::Attributes::new(attrs)?;
    Ok(App {
      ident: ident,
      attrs: attrs,
      fields: fields,
    })
  }
}

impl super::DeriveApp for App {
  fn derive_app(&self, tokens: &mut Tokens) {
    let ident = &self.ident;
    let name = self.attrs.attribute_name();
    let about = self.attrs.attribute_about();
    let settings = self.attrs.attribute_settings();

    tokens.append(quote!{
      impl App for #ident {
        fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
          clap::App::new(#name)
        }

        fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
          app.about(#about)
            .settings(&[ #(#settings),* ])
        }
      }
    });
  }
}

impl super::DeriveFromArgMatches for App {
   fn derive_from_arg_matches(&self, tokens:&mut Tokens) {
    let ident = &self.ident;
    tokens.append(quote!{
      impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #ident {
        fn from(_: &'b clap::ArgMatches<'a>) -> Self {
          Self::default()
        }
      }
    })
  }
}