use syn::{Attribute, Ident, Lit, MetaItem, NestedMetaItem, Variant};
use quote::Tokens;
use attr;

pub struct Subcommand {
  ident: Ident,
  attrs: attr::Attributes,
  variants: Vec<SubcommandVariant>,
}

impl Subcommand {
  pub fn new(ident: Ident, attrs: Vec<Attribute>, v: Vec<Variant>) -> Result<Subcommand, String> {
    let attrs = attr::Attributes::new(attrs)?;

    let mut variants = Vec::with_capacity(v.len());
    for variant in v {
      variants.push(SubcommandVariant::new(variant)?);
    }

    Ok(Subcommand {
      ident: ident,
      attrs: attrs,
      variants: variants,
    })
  }
}

impl super::DeriveApp for Subcommand {
  fn derive_app(&self, tokens: &mut Tokens) {
    let ident = &self.ident;
    let name = self.attrs.attribute_name();
    let about = self.attrs.attribute_about();
    let settings = self.attrs.attribute_settings();

    let subcommand_bodies = self.variants.iter().map(|v: &SubcommandVariant| {
      let name = v.attrs.attribute_name(ident);
      let about = v.attrs.attribute_about(ident);
      let ty = &v.ty;
      quote!{
        <#ty as App>::append(clap::SubCommand::with_name(#name).about(#about))
      }
    });

    tokens.append(quote!{
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
    });
  }
}

impl super::DeriveFromArgMatches for Subcommand {
   fn derive_from_arg_matches(&self, tokens:&mut Tokens) {
    let ident = &self.ident;
    let names = self.variants.iter().map(|v| v.attrs.attribute_name(ident));
    let variants = self.variants.iter().map(|v| {
      let variant = &v.ident;
      quote!(#ident :: #variant(m.into()))
    });

    tokens.append(quote!{
      impl<'a, 'b:'a> From<&'b clap::ArgMatches<'a>> for #ident {
        fn from(m: &'b clap::ArgMatches<'a>) -> Self {
          match m.subcommand() {
            #( (#names, Some(m)) => #variants, )*
            _ => unreachable!(),
          }
        }
      }
    })
  }
}


struct SubcommandVariant {
  ident: Ident,
  ty: ::syn::Ty,
  attrs: VariantAttributes,
}

impl SubcommandVariant {
  fn new(variant: Variant) -> Result<SubcommandVariant, String> {
    let attrs = VariantAttributes::new(variant.attrs)?;

    let ty = match variant.data {
      ::syn::VariantData::Tuple(ref fields) if fields.len() == 1 => fields[0].ty.clone(),
      _ => return Err("".into()),
    };

    Ok(SubcommandVariant {
      ident: variant.ident,
      attrs: attrs,
      ty: ty,
    })
  }
}


#[derive(Default)]
struct VariantAttributes {
  name: Option<String>,
  help: Option<String>,
}

impl VariantAttributes {
  fn new(attrs: Vec<Attribute>) -> Result<VariantAttributes, String> {
    let mut result = VariantAttributes::default();
    for attr in attrs.into_iter().filter(|attr| attr.name() == "clap") {
      result.lookup_item(attr)?;
    }
    Ok(result)
  }

    fn attribute_name(&self, ident: &Ident) -> String {
    self.name.as_ref().map(|s| s.clone()).unwrap_or_else(|| ident.as_ref().to_lowercase())
  }

  fn attribute_about(&self, ident:&Ident) -> String {
    self.help.as_ref().map(|s| s.clone()).unwrap_or_else(|| format!("{}", ident.as_ref()))
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
