//! [![CI](https://github.com/mexus/unzip-n/actions/workflows/ci.yml/badge.svg)](https://github.com/mexus/unzip-n/actions/workflows/ci.yml)
//! [![crates.io](https://img.shields.io/crates/v/unzip-n.svg)](https://crates.io/crates/unzip-n)
//! [![docs.rs](https://docs.rs/unzip-n/badge.svg)](https://docs.rs/unzip-n)
//!
//! Procedural macro for unzipping iterators-over-`n`-length-tuples into `n` collections.
//!
//! Here's a brief example of what it is capable of:
//!
//! ```
//! use unzip_n::unzip_n;
//!
//! unzip_n!(pub 3);
//! use Unzip3 as _; // Just a verification of the trait name.
//! // Or simply leave the visibility modifier absent for inherited visibility
//! // (which usually means "private").
//! // unzip_n!(3);
//!
//! let v = vec![(1, 2, 3), (4, 5, 6)];
//! let (v1, v2, v3) = v.into_iter().unzip_n_vec();
//!
//! assert_eq!(v1, &[1, 4]);
//! assert_eq!(v2, &[2, 5]);
//! assert_eq!(v3, &[3, 6]);
//! ```
//!
//! Or you could give the trait an explicit name:
//!
//! ```
//! use unzip_n::unzip_n;
//!
//! unzip_n!(pub MyTrait 3);
//! use MyTrait as _; // Just a verification that the trait is generated with the desired name.
//!
//! let v = vec![(1, 2, 3), (4, 5, 6)];
//! let (v1, v2, v3) = v.into_iter().unzip_n_vec();
//!
//! assert_eq!(v1, &[1, 4]);
//! assert_eq!(v2, &[2, 5]);
//! assert_eq!(v3, &[3, 6]);
//! ```
//!
//! Or generate multiple traits at once using a range:
//!
//! ```
//! use unzip_n::unzip_n;
//!
//! unzip_n!(pub 2..=4);  // Generates Unzip2, Unzip3, Unzip4
//!
//! let pairs = vec![(1, 2), (3, 4)];
//! let (v1, v2) = pairs.into_iter().unzip_n_vec();
//! assert_eq!(v1, &[1, 3]);
//! assert_eq!(v2, &[2, 4]);
//!
//! let triples = vec![(1, 2, 3), (4, 5, 6)];
//! let (v1, v2, v3) = triples.into_iter().unzip_n_vec();
//! assert_eq!(v1, &[1, 4]);
//! assert_eq!(v2, &[2, 5]);
//! assert_eq!(v3, &[3, 6]);
//! ```
//!
//! With an explicit trait name prefix:
//!
//! ```
//! use unzip_n::unzip_n;
//!
//! unzip_n!(pub MyUnzip 2..4);  // Generates MyUnzip2, MyUnzip3
//! use MyUnzip2 as _;
//! use MyUnzip3 as _;
//! ```
//!
//! # License
//!
//! Licensed under either of
//!
//! * Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! # Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
//! additional terms or conditions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream, discouraged::Speculative},
    parse_macro_input,
    spanned::Spanned,
};

struct UnzipN {
    range: Range,
    trait_name: syn::Ident,
    explicit_name: bool,
    visibility: syn::Visibility,
}

fn int_from_expr(expr: &syn::Expr) -> parse::Result<usize> {
    match expr {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Int(lit_int) => lit_int.base10_parse(),
            _ => {
                let span = expr_lit.span();
                Err(syn::Error::new(span, "Only integer literals are supported"))
            }
        },
        _ => {
            let span = expr.span();
            Err(syn::Error::new(span, "Only integer literals are supported"))
        }
    }
}

fn extract_range(range: syn::ExprRange) -> parse::Result<Range> {
    let start = range
        .start
        .as_ref()
        .map(|start| int_from_expr(start))
        .transpose()?
        .unwrap_or(2)
        .max(2);
    let end = range.end.as_ref().ok_or_else(|| {
        let span = range.span();
        syn::Error::new(span, "Range must be terminated explicitly")
    })?;
    let end = int_from_expr(end)?;
    Ok(match range.limits {
        syn::RangeLimits::HalfOpen(..) => Range::Exclusive(start..end),
        syn::RangeLimits::Closed(..) => Range::Inclusive(start..=end),
    })
}

#[derive(Clone, Debug)]
enum Range {
    Inclusive(std::ops::RangeInclusive<usize>),
    Exclusive(std::ops::Range<usize>),
}

enum RangeIter {
    Inclusive(std::ops::RangeInclusive<usize>),
    Exclusive(std::ops::Range<usize>),
}

impl IntoIterator for &Range {
    type Item = usize;
    type IntoIter = RangeIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Range::Inclusive(range_inclusive) => RangeIter::Inclusive(range_inclusive.clone()),
            Range::Exclusive(range) => RangeIter::Exclusive(range.clone()),
        }
    }
}

impl Iterator for RangeIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RangeIter::Inclusive(range_inclusive) => range_inclusive.next(),
            RangeIter::Exclusive(range) => range.next(),
        }
    }
}

impl Range {
    fn single(&self) -> Option<usize> {
        match self {
            Range::Inclusive(range_inclusive) => {
                if range_inclusive.start() == range_inclusive.end() {
                    Some(*range_inclusive.start())
                } else {
                    None
                }
            }
            Range::Exclusive(range) => {
                if range.start + 1 == range.end {
                    Some(range.start)
                } else {
                    None
                }
            }
        }
    }
}

impl Parse for UnzipN {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let visibility = input.parse()?;

        let maybe_name = if input.peek(syn::Ident) {
            Some(input.parse::<syn::Ident>()?)
        } else {
            None
        };

        let range = {
            let fork = input.fork();
            if let Ok(range_expr) = fork.parse::<syn::ExprRange>() {
                input.advance_to(&fork);
                extract_range(range_expr)?
            } else {
                let n: usize = input.parse::<syn::LitInt>()?.base10_parse()?;
                Range::Inclusive(n..=n)
            }
        };

        let (trait_name, explicit_name) = match maybe_name {
            Some(name) => (name, true),
            None => (format_ident!("Unzip"), false),
        };
        Ok(UnzipN {
            range,
            trait_name,
            explicit_name,
            visibility,
        })
    }
}

impl UnzipN {
    fn make_trait(&self, n: usize, no_std: bool, trait_name: &syn::Ident) -> TokenStream {
        let trait_doc =
            format!("Extension trait for unzipping iterators over tuples of size {n}.",);

        let generic_types = (0..n)
            .map(|id| format_ident!("Type_{id}"))
            .collect::<Vec<_>>();
        let collections = (0..n)
            .map(|id| format_ident!("Collection_{id}"))
            .collect::<Vec<_>>();

        let visibility = &self.visibility;

        let unzip_n_vec = if no_std {
            TokenStream::new()
        } else {
            quote!(
                /// Unzips an iterator over tuples into a tuple of vectors.
                fn unzip_n_vec(self) -> ( #( Vec< #generic_types >, )* )
                where
                    Self: Sized,
                {
                    self.unzip_n()
                }
            )
        };

        quote!(
            #[doc = #trait_doc]
            #visibility trait #trait_name < #( #generic_types, )* > {
                /// Unzips an iterator over tuples into a tuple of collections.
                fn unzip_n<#(#collections,)*>(self) -> ( #(#collections,)* )
                    where
                        #( #collections: Default + Extend< #generic_types >, )*
                        ;

                #unzip_n_vec
            }
        )
    }

    fn make_implementation(&self, n: usize, trait_name: &syn::Ident) -> TokenStream {
        let generic_types = (0..n)
            .map(|id| format_ident!("Type_{id}"))
            .collect::<Vec<_>>();
        let collections = (0..n)
            .map(|id| format_ident!("Collection_{id}"))
            .collect::<Vec<_>>();

        let containers: Vec<_> = (0..n).map(|id| format_ident!("container_{}", id)).collect();
        let values: Vec<_> = (0..n).map(|id| format_ident!("val_{}", id)).collect();

        quote!(
            impl<Iter, #(#generic_types,)*> #trait_name <#(#generic_types,)*> for Iter
            where
                Iter: Iterator<Item = (#(#generic_types,)*)>,
            {
                fn unzip_n<#(#collections,)*>(self) -> ( #(#collections,)* )
                    where
                        #( #collections: Default + Extend< #generic_types >, )*
                {
                    #( let mut #containers = #collections :: default() ;)*
                    self.for_each(|( #( #values, )* )| {
                        #( #containers.extend(std::iter::once(#values)) ;)*
                    });
                    (#( #containers, )* )
                }
            }
        )
    }

    pub fn generate(&self, no_std: bool) -> TokenStream {
        if let Some(single) = self.range.single() {
            // For single values: append number only if no explicit name was provided
            let trait_name = if self.explicit_name {
                self.trait_name.clone()
            } else {
                format_ident!("{}{single}", self.trait_name)
            };
            let trait_decl = self.make_trait(single, no_std, &trait_name);
            let impl_block = self.make_implementation(single, &trait_name);
            quote!( #trait_decl #impl_block )
        } else {
            let mut tokens = TokenStream::new();
            for n in &self.range {
                let trait_name = format_ident!("{}{n}", self.trait_name);
                tokens.extend(self.make_trait(n, no_std, &trait_name));
                tokens.extend(self.make_implementation(n, &trait_name));
            }
            tokens
        }
    }
}

/// Generates a trait and its implementation to "unzip" an iterator of `N`-sized tuple into `N`
/// collections. Accepts either a single number or a range:
///
/// ```
/// # use std::collections::HashSet;
/// # use unzip_n::unzip_n;
/// // Note that visiblity modifier is accepted!
/// unzip_n!(pub(crate) 2);
/// unzip_n!(5);
/// unzip_n!(3);
///
/// let v = vec![(1, 2), (3, 4)];
/// let (s1, s2): (HashSet<_>, HashSet<_>) = v.clone().into_iter().unzip_n();
/// println!("{:?}, {:?}", s1, s2);
///
/// let (v1, v2) = v.into_iter().unzip_n_vec();
/// println!("{:?}, {:?}", v1, v2);
///
/// let v = vec![(1, 2, 3, 4, 5), (6, 7, 8, 9, 10)];
/// let (v1, v2, v3, v4, v5) = v.into_iter().unzip_n_vec();
/// println!("{:?}, {:?}, {:?}, {:?}, {:?}", v1, v2, v3, v4, v5);
/// ```
///
/// ## Range syntax
///
/// You can generate multiple traits at once using range syntax:
///
/// ```
/// # use unzip_n::unzip_n;
/// // Half-open range: generates Unzip2 and Unzip3
/// unzip_n!(2..4);
/// ```
///
/// ```
/// # use unzip_n::unzip_n;
/// // Inclusive range: generates Unzip2, Unzip3, and Unzip4
/// unzip_n!(2..=4);
/// ```
///
/// ```
/// # use unzip_n::unzip_n;
/// // With explicit trait name prefix
/// unzip_n!(pub MyTrait 2..4);  // Generates MyTrait2, MyTrait3
/// ```
///
/// ```
/// # use unzip_n::unzip_n;
/// // Unbounded start defaults to 2 (minimum useful tuple size)
/// unzip_n!(..5);  // Generates Unzip2, Unzip3, Unzip4
/// ```
///
/// For example, `unzip_n(3)` will produce a code like the following:
///
/// ```
/// ///Extension trait for unzipping iterators over tuples of size 3.
/// trait Unzip3<Type_0, Type_1, Type_2> {
///     /// Unzips an iterator over tuples into a tuple of collections.
///     fn unzip_n<Collection_0, Collection_1, Collection_2>(
///         self,
///     ) -> (Collection_0, Collection_1, Collection_2)
///     where
///         Collection_0: Default + Extend<Type_0>,
///         Collection_1: Default + Extend<Type_1>,
///         Collection_2: Default + Extend<Type_2>;
///     /// Unzips an iterator over tuples into a tuple of vectors.
///     fn unzip_n_vec(self) -> (Vec<Type_0>, Vec<Type_1>, Vec<Type_2>)
///     where
///         Self: Sized,
///     {
///         self.unzip_n()
///     }
/// }
///
/// impl<Iter, Type_0, Type_1, Type_2> Unzip3<Type_0, Type_1, Type_2> for Iter
/// where
///     Iter: Iterator<Item = (Type_0, Type_1, Type_2)>,
/// {
///     fn unzip_n<Collection_0, Collection_1, Collection_2>(
///         self,
///     ) -> (Collection_0, Collection_1, Collection_2)
///     where
///         Collection_0: Default + Extend<Type_0>,
///         Collection_1: Default + Extend<Type_1>,
///         Collection_2: Default + Extend<Type_2>,
///     {
///         let mut container_0 = Collection_0::default();
///         let mut container_1 = Collection_1::default();
///         let mut container_2 = Collection_2::default();
///         self.for_each(|(val_0, val_1, val_2)| {
///             container_0.extend(Some(val_0));
///             container_1.extend(Some(val_1));
///             container_2.extend(Some(val_2));
///         });
///         (container_0, container_1, container_2)
///     }
/// }
/// ```
#[proc_macro]
pub fn unzip_n(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as UnzipN).generate(false).into()
}

/// A *no-std* version of the `unzip_n` macro, i.e. without the `unzip_n_vec` trait method.
#[proc_macro]
pub fn unzip_n_nostd(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as UnzipN).generate(true).into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_half_open_range() {
        let range: syn::ExprRange = syn::parse2(quote! { 4..6 }).unwrap();
        let result = extract_range(range).unwrap();
        assert!(matches!(result, Range::Exclusive(r) if r == (4..6)));
    }

    #[test]
    fn test_closed_range() {
        let range: syn::ExprRange = syn::parse2(quote! { 2..=5 }).unwrap();
        let result = extract_range(range).unwrap();
        assert!(matches!(result, Range::Inclusive(r) if r == (2..=5)));
    }

    #[test]
    fn test_unbounded_start_defaults_to_2() {
        let range: syn::ExprRange = syn::parse2(quote! { ..10 }).unwrap();
        let result = extract_range(range).unwrap();
        assert!(matches!(result, Range::Exclusive(r) if r.start == 2 && r.end == 10));
    }

    #[test]
    fn test_start_below_2_clamps_to_2() {
        let range: syn::ExprRange = syn::parse2(quote! { 0..5 }).unwrap();
        let result = extract_range(range).unwrap();
        assert!(matches!(result, Range::Exclusive(r) if r.start == 2));
    }

    #[test]
    fn test_unbounded_end_errors() {
        let range: syn::ExprRange = syn::parse2(quote! { 4.. }).unwrap();
        let result = extract_range(range);
        assert!(result.is_err());
    }

    #[test]
    fn check_parse_unzip_n() {
        let _unzip_n: UnzipN = syn::parse2(quote! { pub 5 }).unwrap();
    }

    // Parsing tests for explicit trait names
    #[test]
    fn parse_single_number_no_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { 5 }).unwrap();
        assert!(!unzip_n.explicit_name);
        assert_eq!(unzip_n.trait_name, "Unzip");
        assert!(matches!(unzip_n.range, Range::Inclusive(r) if r == (5..=5)));
    }

    #[test]
    fn parse_single_number_with_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { MyTrait 5 }).unwrap();
        assert!(unzip_n.explicit_name);
        assert_eq!(unzip_n.trait_name, "MyTrait");
        assert!(matches!(unzip_n.range, Range::Inclusive(r) if r == (5..=5)));
    }

    #[test]
    fn parse_range_no_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { 2..5 }).unwrap();
        assert!(!unzip_n.explicit_name);
        assert_eq!(unzip_n.trait_name, "Unzip");
        assert!(matches!(unzip_n.range, Range::Exclusive(r) if r == (2..5)));
    }

    #[test]
    fn parse_range_with_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { MyTrait 2..5 }).unwrap();
        assert!(unzip_n.explicit_name);
        assert_eq!(unzip_n.trait_name, "MyTrait");
        assert!(matches!(unzip_n.range, Range::Exclusive(r) if r == (2..5)));
    }

    // Visibility modifier tests
    #[test]
    fn parse_pub_visibility() {
        let unzip_n: UnzipN = syn::parse2(quote! { pub 5 }).unwrap();
        assert!(matches!(unzip_n.visibility, syn::Visibility::Public(_)));
    }

    #[test]
    fn parse_pub_crate_visibility() {
        let unzip_n: UnzipN = syn::parse2(quote! { pub(crate) 5 }).unwrap();
        assert!(matches!(unzip_n.visibility, syn::Visibility::Restricted(_)));
    }

    #[test]
    fn parse_inherited_visibility() {
        let unzip_n: UnzipN = syn::parse2(quote! { 5 }).unwrap();
        assert!(matches!(unzip_n.visibility, syn::Visibility::Inherited));
    }

    #[test]
    fn parse_pub_with_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { pub MyTrait 5 }).unwrap();
        assert!(matches!(unzip_n.visibility, syn::Visibility::Public(_)));
        assert!(unzip_n.explicit_name);
        assert_eq!(unzip_n.trait_name, "MyTrait");
    }

    // Generation tests for trait naming
    #[test]
    fn generate_single_no_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { 3 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        // Should generate "Unzip3" trait
        assert!(
            output.contains("trait Unzip3"),
            "Expected 'trait Unzip3' in output: {}",
            output
        );
    }

    #[test]
    fn generate_single_with_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { MyTrait 3 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        // Should generate "MyTrait" trait (no number suffix)
        assert!(
            output.contains("trait MyTrait <"),
            "Expected 'trait MyTrait' in output: {}",
            output
        );
        assert!(
            !output.contains("trait MyTrait3"),
            "Should not contain 'trait MyTrait3' in output: {}",
            output
        );
    }

    #[test]
    fn generate_range_no_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { 2..4 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        // Should generate "Unzip2" and "Unzip3" traits
        assert!(
            output.contains("trait Unzip2"),
            "Expected 'trait Unzip2' in output: {}",
            output
        );
        assert!(
            output.contains("trait Unzip3"),
            "Expected 'trait Unzip3' in output: {}",
            output
        );
        assert!(
            !output.contains("trait Unzip4"),
            "Should not contain 'trait Unzip4' in output: {}",
            output
        );
    }

    #[test]
    fn generate_range_with_explicit_name() {
        let unzip_n: UnzipN = syn::parse2(quote! { MyTrait 2..4 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        // Should generate "MyTrait2" and "MyTrait3" traits
        assert!(
            output.contains("trait MyTrait2"),
            "Expected 'trait MyTrait2' in output: {}",
            output
        );
        assert!(
            output.contains("trait MyTrait3"),
            "Expected 'trait MyTrait3' in output: {}",
            output
        );
        assert!(
            !output.contains("trait MyTrait4"),
            "Should not contain 'trait MyTrait4' in output: {}",
            output
        );
    }

    #[test]
    fn generate_inclusive_range() {
        let unzip_n: UnzipN = syn::parse2(quote! { 2..=3 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        // Should generate "Unzip2" and "Unzip3" traits (inclusive)
        assert!(
            output.contains("trait Unzip2"),
            "Expected 'trait Unzip2' in output: {}",
            output
        );
        assert!(
            output.contains("trait Unzip3"),
            "Expected 'trait Unzip3' in output: {}",
            output
        );
    }

    #[test]
    fn generate_pub_visibility() {
        let unzip_n: UnzipN = syn::parse2(quote! { pub 3 }).unwrap();
        let output = unzip_n.generate(false).to_string();
        assert!(
            output.contains("pub trait Unzip3"),
            "Expected 'pub trait Unzip3' in output: {}",
            output
        );
    }
}
