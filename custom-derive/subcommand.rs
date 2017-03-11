use syn::{Attribute, Body, DeriveInput, Ident, Lit, MetaItem, NestedMetaItem, Variant};
use quote::Tokens;


pub struct Subcommand {
  ident: Ident,
  attr: SubcommandAttribute,
  variants: Vec<SubcommandVariant>,
}

impl Subcommand {
  pub fn new(ast: DeriveInput) -> Result<Subcommand, String> {
    let ident = ast.ident;
    let attr = SubcommandAttribute::new(ast.attrs)?;

    match ast.body {
      Body::Enum(_variants) => {
        let mut variants = Vec::with_capacity(_variants.len());
        for variant in _variants {
          variants.push(SubcommandVariant::new(variant)?);
        }

        Ok(Subcommand {
          ident: ident,
          attr: attr,
          variants: variants,
        })
      }
      Body::Struct(_) => Err("#[derive(Subcommand)] is only supported for enum".into()),
    }
  }

  fn attribute_name(&self) -> Tokens {
    self.attr.name.as_ref().map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_NAME")))
  }

  fn attribute_about(&self) -> Tokens {
    self.attr.about.as_ref().map(|s| quote!(#s)).unwrap_or(quote!(env!("CARGO_PKG_DESCRIPTION")))
  }

  pub fn attribute_settings(&self) -> Vec<Tokens> {
    self.attr
      .settings
      .iter()
      .map(|s| match s.as_str() {
        "VersionlessSubcommands" => quote!(::clap::AppSettings::VersionlessSubcommands),
        "SubcommandRequiredElseHelp" => quote!(::clap::AppSettings::SubcommandRequiredElseHelp),
        _ => panic!("invalid setting value"),
      })
      .collect()
  }

  pub fn to_derived_tokens(&self) -> Tokens {
    let mut tokens = Tokens::new();
    tokens.append_all(&[self.to_derived_tokens_subcommand(), self.to_derived_tokens_from()]);
    tokens
  }

  fn to_derived_tokens_subcommand(&self) -> Tokens {
    let ident = &self.ident;
    let name = self.attribute_name();
    let about = self.attribute_about();
    let settings = self.attribute_settings();
    let body = self.variants
      .iter()
      .map(|v| v.to_derived_tokens_app());
    quote! {
      impl Subcommand for #ident {
        fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
          clap::App::new(#name)
            .about(#about)
            .settings(&[ #(#settings),* ])
            #( #body )*
        }
      }
    }
  }

  pub fn to_derived_tokens_from(&self) -> Tokens {
    let ident = &self.ident;
    let body = self.variants
      .iter()
      .map(|v| v.to_derived_tokens_from(&self.ident));
    quote! {
      impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #ident {
        fn from(m: &'b clap::ArgMatches<'a>) -> Self {
          match m.subcommand() {
            #(#body)*
            _ => unreachable!(),
          }
        }
      }
    }
  }
}


#[derive(Default)]
struct SubcommandAttribute {
  name: Option<String>,
  about: Option<String>,
  settings: Vec<String>,
}

impl SubcommandAttribute {
  fn new(attrs: Vec<Attribute>) -> Result<SubcommandAttribute, String> {
    let mut result = SubcommandAttribute::default();
    for attr in attrs {
      result.lookup_item(attr)?;
    }
    Ok(result)
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


struct SubcommandVariant {
  ident: Ident,
  attr: SubcommandVariantAttribute,
}

impl SubcommandVariant {
  fn new(variant: Variant) -> Result<SubcommandVariant, String> {
    let attr = SubcommandVariantAttribute::new(variant.attrs)?;
    Ok(SubcommandVariant {
      ident: variant.ident,
      attr: attr,
    })
  }

  fn attribute_name(&self) -> String {
    self.attr.name.as_ref().map(|s| s.clone()).unwrap_or_else(|| self.ident.as_ref().to_lowercase())
  }

  fn attribute_help(&self) -> String {
    self.attr.help.as_ref().map(|s| s.clone()).unwrap_or_else(|| format!("{}", self.ident.as_ref()))
  }

  fn to_derived_tokens_app(&self) -> Tokens {
    let name = self.attribute_name();
    let about = self.attribute_help();
    quote! {
      .subcommand(clap::SubCommand::with_name(#name).about(#about))
    }
  }

  fn to_derived_tokens_from(&self, ident: &Ident) -> Tokens {
    let name = self.attribute_name();
    let variant = &self.ident;
    quote! {
      (#name, _) => #ident :: #variant(Default::default()),
    }
  }
}


#[derive(Default)]
struct SubcommandVariantAttribute {
  name: Option<String>,
  help: Option<String>,
}

impl SubcommandVariantAttribute {
  fn new(attrs: Vec<Attribute>) -> Result<SubcommandVariantAttribute, String> {
    let mut result = SubcommandVariantAttribute::default();
    for attr in attrs.into_iter().filter(|attr| attr.name() == "clap") {
      result.lookup_item(attr)?;
    }
    Ok(result)
  }

  fn lookup_item(&mut self, attr: Attribute) -> Result<(), String> {
    let items = match attr.value {
      MetaItem::List(_, ref items) => items,
      _ => return Ok(()),
    };

    for item in items {
      let (ident, value) = match *item {
        NestedMetaItem::MetaItem(MetaItem::NameValue(ref ident, Lit::Str(ref value, _))) => {
          (ident, value)
        }
        _ => return Err("Invalid attribute format".into()),
      };

      match ident.as_ref() {
        "name" => self.name = Some(value.to_owned()),
        "help" => self.help = Some(value.to_owned()),
        _ => return Err("No such attribute item".into()),
      }
    }

    Ok(())
  }
}
