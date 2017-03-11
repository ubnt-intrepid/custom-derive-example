use syn::{Attribute, Body, DeriveInput, Ident, Lit, MetaItem, NestedMetaItem, Variant};
use quote::Tokens;
use util;


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

  pub fn to_derived_tokens(&self) -> Tokens {
    let ident = &self.ident;
    let name = &self.attr.name;
    let about = &self.attr.about;

    let subcommand_apps = {
      util::join_tokens(self.variants.iter().map(|v| v.to_derived_tokens_app()))
    };

    let subcommand_froms = {
      util::join_tokens(self.variants.iter().map(|v| v.to_derived_tokens_from(&self.ident)))
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


struct SubcommandAttribute {
  name: Tokens,
  about: Tokens,
}

impl SubcommandAttribute {
  fn new(attrs: Vec<Attribute>) -> Result<SubcommandAttribute, String> {
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

  fn subcommand_name(&self) -> String {
    self.attr.name.as_ref().map(|s| s.clone()).unwrap_or_else(|| self.ident.as_ref().to_lowercase())
  }

  fn subcommand_help(&self) -> String {
    self.attr.help.as_ref().map(|s| s.clone()).unwrap_or_else(|| format!("{}", self.ident.as_ref()))
  }

  fn to_derived_tokens_app(&self) -> Tokens {
    let name = self.subcommand_name();
    let about = self.subcommand_help();
    quote! {
      .subcommand(clap::SubCommand::with_name(#name).about(#about))
    }
  }

  fn to_derived_tokens_from(&self, ident: &Ident) -> Tokens {
    let name = self.subcommand_name();
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