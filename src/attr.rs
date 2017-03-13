use syn::{Attribute, Lit, MetaItem, NestedMetaItem};
use quote;

#[derive(Default)]
pub struct Attributes {
  pub name: Option<String>,
  pub about: Option<String>,
  pub settings: Vec<String>,
}

impl Attributes {
  pub fn new(attrs: Vec<Attribute>) -> Result<Attributes, String> {
    let mut result = Attributes::default();
    for attr in attrs {
      result.lookup_item(attr)?;
    }
    Ok(result)
  }

  pub fn attribute_name(&self) -> quote::Tokens {
    self.name.as_ref().map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_NAME")))
  }

  pub fn attribute_about(&self) -> quote::Tokens {
    self.about.as_ref().map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_DESCRIPTION")))
  }

  pub fn attribute_settings(&self) -> Vec<quote::Tokens> {
    self.settings
      .iter()
      .map(|s| match s.as_str() {
        "VersionlessSubcommands" => quote!(::clap::AppSettings::VersionlessSubcommands),
        "SubcommandRequiredElseHelp" => quote!(::clap::AppSettings::SubcommandRequiredElseHelp),
        _ => panic!("invalid setting value"),
      })
      .collect()
  }

  fn lookup_item(&mut self, attr: Attribute) -> Result<(), String> {
    if attr.name() != "clap" {
      return Ok(());
    }

    let items = match attr.value {
      MetaItem::List(_, items) => items,
      _ => return Ok(()),
    };

    for item in items {
      match item { 
        NestedMetaItem::MetaItem(item) => {
          match item {
            MetaItem::NameValue(ref ident, Lit::Str(ref value, _)) => {
              match ident.as_ref() {
                "name" => self.name = Some(value.to_owned()),
                "about" => self.about = Some(value.to_owned()),
                _ => return Err("invalid attribute name".into()),
              }
            }
            MetaItem::Word(ref ident) => self.settings.push(ident.as_ref().into()),
            _ => return Err("invalud attribute".into()),
          }
        }
        _ => return Err("invalud attribute".into()),
      }
    }

    Ok(())
  }
}
