use syn::*;
use syn::spanned::Spanned;

/// Try to parse a boolean value for an attribute option.
pub fn parse_boolean(m: &Meta, opt: &str) -> Result<Option<bool>> {
    match m {
        Meta::Path(p) => {
            if !p.is_ident(opt) {
                return Ok(None);
            }

            Ok(Some(true))
        }
        Meta::NameValue(nv) => {
            if !nv.path.is_ident(opt) {
                return Ok(None);
            }

            match &nv.lit {
                Lit::Bool(b) => Ok(Some(b.value())),
                _ => return Err(Error::new(nv.lit.span(), "expected a boolean literal")),
            }
        }
        Meta::List(l) => {
            if l.path.is_ident(opt) {
                return Err(Error::new(
                    l.span(),
                    "invalid syntax for boolean option: unexpected list",
                ));
            }

            Ok(None)
        }
    }
}

/// Try to parse a string value for an attribute option.
pub fn parse_string(m: &Meta, opt: &str) -> Result<Option<String>> {
    match m {
        Meta::NameValue(nv) => {
            if !nv.path.is_ident(opt) {
                return Ok(None);
            }

            match &nv.lit {
                Lit::Str(s) => Ok(Some(s.value())),
                _ => return Err(Error::new(nv.lit.span(), "expected a string literal")),
            }
        }
        Meta::List(l) => {
            if l.path.is_ident(opt) {
                return Err(Error::new(
                    l.span(),
                    "invalid syntax for string option: unexpected list",
                ));
            }

            Ok(None)
        }
        Meta::Path(p) if p.is_ident(opt) => {
            Err(Error::new(m.span(), "expected a string literal value"))
        }
        _ => Ok(None),
    }
}

/// Try to parse an identifier as a string value in an attribute option.
pub fn parse_ident_as_string(m: &Meta) -> Option<String> {
    match m {
        Meta::Path(p) => p.get_ident().map(Ident::to_string),
        _ => None,
    }
}
