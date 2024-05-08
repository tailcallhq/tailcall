// use crate::blueprint::FieldDefinition;
// use crate::config::{self, ConfigModule, Field};
// use crate::lambda::{Expression, IO};
// use crate::try_fold::TryFold;
// use crate::valid::Valid;

// pub fn update_validate<'a>(
//     type_name: &'a str,
// ) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
// {
//     TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
//         |(config_module, field, _type, _), mut b_field| {
//             if field.validate.is_some()
//                 || _type.validate.is_some()
//                 || config_module
//                     .find_type(&field.type_of)
//                     .and_then(|_type| _type.validate.as_ref())
//                     .is_some()
//             {
//                 if !config_module.is_scalar(type_name) {
//                     return Valid::fail("@validate can only be used on custom scalars".to_owned());
//                 }
//             }
//         }
//     )
// }