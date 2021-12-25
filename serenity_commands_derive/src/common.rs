use proc_macro2::{Ident, Span};
use syn::spanned::Spanned;
use syn::{Attribute, Fields, Lit, Meta, Path, Type, Variant};
use syn::{Error, Result};

pub struct AttrOption<T> {
    name: &'static str,
    value: Option<T>,
}

impl<T> AttrOption<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            value: None,
        }
    }

    pub fn set(&mut self, span: Span, value: T) -> Result<()> {
        if self.value.is_some() {
            return Err(Error::new(
                span,
                format!("`{}` parameter has already been provided", self.name),
            ));
        }

        self.value = Some(value);

        Ok(())
    }

    pub fn value(self) -> Option<T> {
        self.value
    }
}

pub fn get_lit_string(lit: &Lit) -> Result<String> {
    match lit {
        Lit::Str(s) => Ok(s.value()),
        _ => Err(Error::new(lit.span(), "expected a string literal")),
    }
}

#[allow(dead_code)]
pub fn get_lit_boolean(lit: &Lit) -> Result<bool> {
    match lit {
        Lit::Bool(b) => Ok(b.value()),
        _ => Err(Error::new(lit.span(), "expected a boolean literal")),
    }
}

pub fn get_path_as_string(p: &Path) -> Result<String> {
    p.get_ident()
        .map(Ident::to_string)
        .ok_or_else(|| Error::new(p.span(), "expected an identifier"))
}

pub fn parse_doc(attr: &Attribute) -> Result<String> {
    let nv = match attr.parse_meta()? {
        Meta::NameValue(nv) => nv,
        _ => return Err(Error::new(attr.span(), "invalid documentation string")),
    };

    Ok(match nv.lit {
        Lit::Str(s) => s.value().trim().to_string(),
        _ => return Err(Error::new(nv.span(), "expected string")),
    })
}

pub fn is_option(ty: &Type) -> bool {
    match ty {
        // Naive approach, but it's the best we can do. The compiler
        // only provides us tokens, with no other way of retrieving
        // type data, since procedural macros are invoked before
        // type-checking.
        Type::Path(p) => p.path.segments.last().unwrap().ident == "Option",
        _ => false,
    }
}

pub fn ensure_tuple_variant(variant: &Variant, entity: &str) -> Result<()> {
    match &variant.fields {
        Fields::Unnamed(n) if n.unnamed.len() != 1 => Err(Error::new(
            n.span(),
            format_args!("expected a single {} as a field of this tuple struct variant", entity),
        )),
        Fields::Unnamed(_) => Ok(()),
        _ => {
            let mut err = Error::new(
                variant.span(),
                format_args!("expected a {} as a field in a tuple struct variant", entity),
            );

            err.combine(Error::new(
                variant.span(),
                format_args!("note: try changing this to `{0}({0})`", variant.ident),
            ));

            Err(err)
        },
    }
}
