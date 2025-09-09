use syn::{Error, GenericArgument, PathArguments, Type};

use crate::{
    entity::EntityField,
    relations::{Relation, RelationType},
};

pub fn validate_relations(rels: &[Relation], fields: &[EntityField]) -> syn::Result<()> {
    for rel in rels {
        let required_field = &rel.relation_name;
        let found = fields
            .iter()
            .find(|f| &f.ident.to_string() == required_field);

        if let Some(field) = found {
            if !field.is_ignored() {
                return Err(syn::Error::new_spanned(
                    &field.ident,
                    "Ignore this field with #[sqlx(skip)] attribute",
                ));
            }

            if let Type::Path(path) = &field.ty {
                let path_seg = path.path.segments.last().unwrap();

                // Must be Option<...>
                if path_seg.ident != "Option" {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "This field has to be optional; hint: wrap in Option<T>",
                    ));
                }

                // Extract Option<T>
                if let PathArguments::AngleBracketed(args) = &path_seg.arguments
                    && let Some(GenericArgument::Type(inner_ty)) = args.args.first()
                    && let Type::Path(inner_path) = inner_ty
                {
                    let inner_seg = inner_path.path.segments.last().unwrap();
                    match &rel.kind {
                        RelationType::HasMany => {
                            if inner_seg.ident != "Vec" {
                                return Err(syn::Error::new_spanned(
                                    inner_ty,
                                    format!(
                                        "Expected Vec<{}> inside Option because of HasMany",
                                        &rel.other
                                    ),
                                ));
                            }
                        }
                        RelationType::HasOne => {
                            if inner_seg.ident != rel.other {
                                return Err(Error::new_spanned(
                                    inner_ty,
                                    format!(
                                        "Expected {} inside Option because of HasOne",
                                        &rel.other
                                    ),
                                ));
                            }
                        }
                        RelationType::BelongsTo => {
                            if inner_seg.ident != rel.other {
                                return Err(Error::new_spanned(
                                    inner_ty,
                                    format!(
                                        "Expected {} inside Option because of BelongsTo",
                                        &rel.other
                                    ),
                                ));
                            }
                        }
                    }

                    //
                    // // Extract Vec<T>
                    // if let PathArguments::AngleBracketed(vec_args) = &inner_seg.arguments {
                    //     if let Some(GenericArgument::Type(final_ty)) = vec_args.args.first()
                    //     {
                    //         // final_ty is Jar
                    //         if let Type::Path(final_path) = final_ty {
                    //             let final_seg = final_path.path.segments.last().unwrap();
                    //
                    //             if final_seg.ident != "Jar" {
                    //                 return Err(Error::new_spanned(
                    //                     final_ty,
                    //                     "Expected Jar inside Vec",
                    //                 ));
                    //             }
                    //         }
                    //     }
                    // }
                }
            }
        } else {
            let expected_ty = match rel.kind {
                RelationType::HasMany => format!("Option<Vec<{}>>", rel.other),
                RelationType::HasOne => format!("Option<{}>", rel.other),
                RelationType::BelongsTo => format!("Option<{}>", rel.other),
            };

            return Err(Error::new_spanned(
                &rel.on.0,
                format!(
                    "You must create a field `{}` of type `{}` and mark it with #[sql(skip)]",
                    rel.relation_name, expected_ty
                ),
            ));
        }
    }
    Ok(())
}
