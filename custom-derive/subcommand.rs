use syn::{Attribute, Ident, Lit, MetaItem, NestedMetaItem, Variant};
use quote::Tokens;


pub struct Subcommand {
  ident: Ident,
  attr: SubcommandAttribute,
  variants: Vec<SubcommandVariant>,
}

impl Subcommand {
  pub fn new(ident: Ident, attrs: Vec<Attribute>, v: Vec<Variant>) -> Result<Subcommand, String> {
    let attrs = SubcommandAttribute::new(attrs)?;

    let mut variants = Vec::with_capacity(v.len());
    for variant in v {
      variants.push(SubcommandVariant::new(variant)?);
    }

    Ok(Subcommand {
      ident: ident,
      attr: attrs,
      variants: variants,
    })
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

  fn to_derived_tokens_subcommand(&self) -> Tokens {
    let ident = &self.ident;
    let name = self.attribute_name();
    let about = self.attribute_about();
    let settings = self.attribute_settings();
    let subcommand_bodies = self.variants.iter().map(|v: &SubcommandVariant| {
      let name = v.attribute_name();
      let about = v.attribute_about();
      let ty = &v.ty;
      quote!{
        <#ty as App>::append(clap::SubCommand::with_name(#name).about(#about))
      }
    });

    quote![
      impl App for #ident {
        fn app<'a, 'b: 'a>() -> clap::App<'a, 'b> {
          clap::App::new(#name)
        }

        fn append<'a, 'b: 'a>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
          app.about(#about)
            .settings(&[ #(#settings),* ])
            #( .subcommand(#subcommand_bodies) )*
        }
      }
    ]
  }

  pub fn to_derived_tokens_from(&self) -> Tokens {
    let ident = &self.ident;
    let names = self.variants.iter().map(|v| v.attribute_name());
    let variants = self.variants.iter().map(|v| {
      let variant = &v.ident;
      quote!(#ident :: #variant(m.into()))
    });
    quote![
      impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #ident {
        fn from(m: &'b clap::ArgMatches<'a>) -> Self {
          match m.subcommand() {
            #( (#names, Some(m)) => #variants, )*
            _ => unreachable!(),
          }
        }
      }
    ]
  }
}

impl ::quote::ToTokens for Subcommand {
  fn to_tokens(&self, tokens: &mut Tokens) {
    tokens.append_all(&[self.to_derived_tokens_subcommand(), self.to_derived_tokens_from()]);
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
  ty: ::syn::Ty,
  attr: SubcommandVariantAttribute,
}

impl SubcommandVariant {
  fn new(variant: Variant) -> Result<SubcommandVariant, String> {
    let attr = SubcommandVariantAttribute::new(variant.attrs)?;

    let ty = match variant.data {
      ::syn::VariantData::Tuple(ref fields) if fields.len() == 1 => fields[0].ty.clone(),
      _ => return Err("".into()),
    };

    Ok(SubcommandVariant {
      ident: variant.ident,
      attr: attr,
      ty: ty,
    })
  }

  fn attribute_name(&self) -> String {
    self.attr.name.as_ref().map(|s| s.clone()).unwrap_or_else(|| self.ident.as_ref().to_lowercase())
  }

  fn attribute_about(&self) -> String {
    self.attr.help.as_ref().map(|s| s.clone()).unwrap_or_else(|| format!("{}", self.ident.as_ref()))
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
