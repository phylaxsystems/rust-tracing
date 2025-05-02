use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, LitStr};

/// A parsed Field of a struct
pub(crate) struct Field {
    env_var: Option<LitStr>,
    field_name: Option<Ident>,
    field_type: syn::Type,

    optional: bool,
    infallible: bool,
    skip: bool,
    desc: Option<String>,

    _attrs: Vec<syn::Attribute>,

    span: proc_macro2::Span,
}

impl TryFrom<&syn::Field> for Field {
    type Error = syn::Error;

    fn try_from(field: &syn::Field) -> Result<Self, syn::Error> {
        let mut optional = false;
        let mut env_var = None;
        let mut infallible = false;
        let mut desc = None;
        let mut skip = false;

        field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("from_env"))
            .for_each(|attr| {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("skip") {
                        skip = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("optional") {
                        optional = true;
                        return Ok(());
                    }
                    if meta.path.is_ident("var") {
                        env_var = Some(meta.value()?.parse::<LitStr>()?);
                        return Ok(());
                    }
                    if meta.path.is_ident("desc") {
                        desc = Some(meta.value()?.parse::<LitStr>()?.value());
                        return Ok(());
                    }
                    if meta.path.is_ident("infallible") {
                        infallible = true;
                    }
                    Ok(())
                });
            });

        if desc.is_none() && env_var.is_some() {
            return Err(syn::Error::new(
                field.span(),
                "Missing description for field. Use `#[from_env(desc = \"DESC\")]`",
            ));
        }

        let field_type = field.ty.clone();
        let field_name = field.ident.clone();
        let span = field.span();

        Ok(Field {
            env_var,
            field_name,
            field_type,
            optional,
            skip,
            infallible,
            desc,
            _attrs: field
                .attrs
                .iter()
                .filter(|attr| !attr.path().is_ident("from_env"))
                .cloned()
                .collect(),
            span,
        })
    }
}

impl Field {
    pub(crate) fn trait_name(&self) -> TokenStream {
        self.env_var
            .as_ref()
            .map(|_| quote! { FromEnvVar })
            .unwrap_or(quote! { FromEnv })
    }

    pub(crate) fn as_trait(&self) -> TokenStream {
        let field_trait = self.trait_name();
        let field_type = &self.field_type;

        quote! { <#field_type as #field_trait> }
    }

    pub(crate) fn assoc_err(&self) -> TokenStream {
        let as_trait = self.as_trait();

        quote! { #as_trait::Error }
    }

    pub(crate) fn field_name(&self, idx: usize) -> Ident {
        if let Some(field_name) = self.field_name.as_ref() {
            return field_name.clone();
        }

        let n = format!("field_{}", idx);
        syn::parse_str::<Ident>(&n)
            .map_err(|_| syn::Error::new(self.span, "Failed to create field name"))
            .unwrap()
    }

    /// Produces the name of the enum variant for the field
    pub(crate) fn enum_variant_name(&self, idx: usize) -> Option<TokenStream> {
        if self.skip || self.infallible {
            return None;
        }

        let n = self.field_name(idx).to_string().to_pascal_case();

        let n: Ident = syn::parse_str::<Ident>(&n)
            .map_err(|_| syn::Error::new(self.span, "Failed to create field name"))
            .unwrap();

        Some(quote! { #n })
    }

    /// Produces the variant, containing the error type
    pub(crate) fn expand_enum_variant(&self, idx: usize) -> Option<TokenStream> {
        let variant_name = self.enum_variant_name(idx)?;
        let var_name_str = variant_name.to_string();
        let assoc_err = self.assoc_err();

        Some(quote! {
            #[doc = "Error for "]
            #[doc = #var_name_str]
            #variant_name(#assoc_err)
        })
    }

    /// Produces the a line for the `inventory` function
    /// of the form
    /// items.push(...); // (if this is a FromEnvVar)
    /// or
    /// items.extend(...); // (if this is a FromEnv)
    /// or
    /// // nothing if this is a skip
    pub(crate) fn expand_env_item_info(&self) -> TokenStream {
        if self.skip {
            return quote! {};
        }

        let description = self.desc.clone().unwrap_or_default();
        let optional = self.optional;

        if let Some(env_var) = &self.env_var {
            let var_name = env_var.value();

            return quote! {
                items.push(&EnvItemInfo {
                    var: #var_name,
                    description: #description,
                    optional: #optional,
                });
            };
        }

        let field_ty = &self.field_type;
        quote! {
            items.extend(
                <#field_ty as FromEnv>::inventory()
            );
        }
    }

    pub(crate) fn expand_variant_display(&self, idx: usize) -> Option<TokenStream> {
        let variant_name = self.enum_variant_name(idx)?;

        Some(quote! {
            Self::#variant_name(err) => err.fmt(f)
        })
    }

    pub(crate) fn expand_variant_source(&self, idx: usize) -> Option<TokenStream> {
        let variant_name = self.enum_variant_name(idx)?;

        Some(quote! {
            Self::#variant_name(err) => Some(err)
        })
    }

    pub(crate) fn expand_item_from_env(&self, err_ident: &Ident, idx: usize) -> TokenStream {
        // Produces code fo the following form:
        // ```rust
        // // EITHER
        // let field_name = env::var(#self.env_var.unwrap()).map_err(|e| e.map(#ErroEnum::FieldName))?;

        // // OR
        // let field_name =  FromEnvVar::from_env_var(#self.env_var.unwrap()).map_err(|e| e.map(#ErroEnum::FieldName))?;

        // // OR
        // let field_name =  FromEnv::from_env().map_err()?;

        // // OR
        // let field_name = Default::default();

        //```
        let variant = self.enum_variant_name(idx);
        let field_name = self.field_name(idx);

        if self.skip {
            return quote! {
                let #field_name = Default::default();
            };
        }

        let fn_invoc = if let Some(ref env_var) = self.env_var {
            quote! { FromEnvVar::from_env_var(#env_var) }
        } else {
            quote! { FromEnv::from_env() }
        };

        let map_line = if self.infallible {
            quote! { FromEnvErr::infallible_into }
        } else {
            quote! { |e| e.map(#err_ident::#variant) }
        };

        quote! {
            let #field_name = #fn_invoc
                .map_err(#map_line)?;
        }
    }
}
