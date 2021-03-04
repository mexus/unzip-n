//! Extended macros parameters helpers.

use syn::parse::Parse;

/// Extended trait configuration (trait name, attributes).
pub struct ExtendedConfiguration {
    /// User-provided trait name.
    pub trait_name: syn::Ident,

    /// Trait attributes.
    pub attributes: Vec<syn::Attribute>,

    /// Visibility of the generated trait.
    pub visibility: syn::Visibility,
}

impl Parse for ExtendedConfiguration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse()?;
        input.parse::<syn::Token!(trait)>()?;
        let trait_name = input.parse()?;
        Ok(ExtendedConfiguration {
            trait_name,
            attributes,
            visibility,
        })
    }
}

#[cfg(test)]
mod test {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn check_extended() {
        let input = r#"
/// First line
#[doc = "second line"]
#[cfg(test)]
pub trait SomeName"#;
        let ExtendedConfiguration {
            attributes,
            trait_name,
        } = syn::parse_str(input).unwrap();
        assert_eq!(trait_name.to_string(), "SomeName");
        assert_eq!(
            attributes[0].to_token_stream().to_string(),
            quote::quote! {#[doc = " First line"]}.to_string()
        );
        assert_eq!(
            attributes[1].to_token_stream().to_string(),
            quote::quote! {#[doc = "second line"]}.to_string()
        );
        assert_eq!(
            attributes[2].to_token_stream().to_string(),
            quote::quote! {#[cfg(test)]}.to_string()
        );
        assert_eq!(attributes.len(), 3);
    }
}
