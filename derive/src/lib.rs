extern crate proc_macro;

use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, parse_macro_input, Attribute, AttrStyle, DeriveInput, Data,
    DataStruct, Fields, Type,
};

// most of this code comes directly from bytemuck_derive, with slight modifications

fn get_ident_from_stream(tokens: TokenStream) -> Option<Ident> {
    match tokens.into_iter().next() {
        Some(TokenTree::Group(group)) => get_ident_from_stream(group.stream()),
        Some(TokenTree::Ident(ident)) => Some(ident),
        _ => None,
    }
}

fn get_attr(attributes: &[Attribute], attr_name: &str) -> Option<Ident> {
    for attr in attributes {
        if let (
            AttrStyle::Outer,
            Some(outer_ident),
            Some(inner_ident)
        ) = (
            &attr.style,
            attr.path.get_ident(),
            get_ident_from_stream(attr.tokens.clone())
        ) {
            if outer_ident.to_string() == attr_name { return Some(inner_ident); }
        }
    }

    None
}

fn verify_attributes(attributes: &[Attribute]) -> Result<(), &'static str> {
    let repr_attr = get_attr(attributes, "repr");
    let error_str = "Castable requires #[repr(C)], #[repr(transparent), #[repr(packed)] or #[repr(align)]";

    match repr_attr {
        Some(ident) => {
            let repr_string = ident.to_string();

            match repr_string.as_str() {
                "C" => Ok(()),
                "transparent" => Ok(()),
                "packed" => Ok(()),
                "align" => Ok(()),
                _ => Err(error_str),
            }
        },
        None => Err(error_str)
    }
}

fn get_struct_fields(input: &DeriveInput) -> Result<&Fields, &'static str> {
    if let Data::Struct(DataStruct { fields, .. }) = &input.data {
        Ok(fields)
    } else {
        Err("deriving this trait is only supported for structs")
    }
}

fn get_field_types<'a>(
    fields: &'a Fields,
) -> impl Iterator<Item = &'a Type> + 'a {
    fields.iter().map(|field| &field.ty)
}

fn generate_assert_no_padding(
    input: &DeriveInput,
) -> Result<TokenStream, &'static str> {
    let struct_type = &input.ident;
    let span = input.ident.span();
    let fields = get_struct_fields(input)?;

    let mut field_types = get_field_types(&fields);
    let size_sum = if let Some(first) = field_types.next() {
        let size_first = quote_spanned!(span => ::std::mem::size_of::<#first>());
        let size_rest =
            quote_spanned!(span => #( + ::std::mem::size_of::<#field_types>() )*);

        quote_spanned!(span => #size_first#size_rest)
    } else {
        quote_spanned!(span => 0)
    };

    Ok(quote_spanned! {span => const _: fn() = || {
        struct TypeWithoutPadding([u8; #size_sum]);
        let _ = ::std::mem::transmute::<#struct_type, TypeWithoutPadding>;
    };})
}

fn generate_assert_castable(
  input: &DeriveInput
) -> Result<TokenStream, &'static str> {
    let trait_ = quote!(::pkbuffer::Castable);
    // let (impl_generics, _ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = get_struct_fields(input)?;
    let span = input.span();
    let field_types = get_field_types(&fields);
    
    Ok(quote_spanned! {span => #(const _: fn() = || {
        fn check() {
            fn assert_impl<T: #trait_>() {}
            assert_impl::<#field_types>();
        }
    };)*})
}

fn verify_struct_members(input: &DeriveInput) -> Result<TokenStream, &'static str> {
    if !input.generics.params.is_empty() {
        return Err("Castable cannot be derived for structures with generic parameters");
    }

    let assert_no_padding = generate_assert_no_padding(input)?;
    let assert_fields_are_castable = generate_assert_castable(input)?;

    Ok(quote!(
        #assert_no_padding
        #assert_fields_are_castable
    ))
}

fn derive_castable_verify(input: DeriveInput) -> Result<TokenStream, &'static str> {
    let name = &input.ident;
    let castable_trait = quote!(::pkbuffer::Castable);

    verify_attributes(&input.attrs)?;
    let struct_asserts = verify_struct_members(&input)?;

    Ok(quote! {
        #struct_asserts

        unsafe impl #castable_trait for #name {}
    })
}

fn derive_castable_panic(input: DeriveInput) -> TokenStream {
    derive_castable_verify(input).unwrap_or_else(|err| {
        quote! { compile_error!(#err); }
    })
}
    
/// Derive the `Castable` trait for a given object.
///
/// This macro ensures that most of the safety requirements for the `Castable` trait are met:
///
/// * The type does not contain any padding bytes.
/// * The type's members are also `Castable`.
/// * The type is `#[repr(C)]`, `#[repr(transparent)]`, `#[repr(packed)]` or `#[repr(align)]`.
/// * The type must not use generics.
///
/// If one of these traits aren't met, the derive macro will fail.
#[proc_macro_derive(Castable)]
pub fn derive_castable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded = derive_castable_panic(parse_macro_input!(input as DeriveInput));
    
    proc_macro::TokenStream::from(expanded)
}

