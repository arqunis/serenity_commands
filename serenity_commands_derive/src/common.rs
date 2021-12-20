use proc_macro2::{Ident, Span};
use syn::spanned::Spanned;
use syn::{Attribute, Lit, Meta, Path};
use syn::{Error, Result};

pub struct AttrOption<T> {
    name: &'static str,
    value: Option<T>,
}

impl<T> AttrOption<T> {
    pub fn new(name: &'static str) -> Self {
        Self { name, value: None }
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
