#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
mod de {
    use crate::ignore::Ignore;
    use crate::schema::Schema;
    use serde::de;
    use serde_json;
    pub struct Deserialize<'de> {
        schema: &'de Schema,
    }
    impl<'de> Deserialize<'de> {
        pub fn new(schema: &'de Schema) -> Self {
            Self { schema }
        }
    }
    impl<'de> de::DeserializeSeed<'de> for Deserialize<'de> {
        type Value = serde_json::Value;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let visitor = Visitor::new(self.schema);
            match &self.schema {
                Schema::Boolean => deserializer.deserialize_bool(visitor),
                Schema::Number(n) => match n {
                    crate::schema::N::I64 => deserializer.deserialize_i64(visitor),
                    crate::schema::N::U64 => deserializer.deserialize_u64(visitor),
                    crate::schema::N::F64 => deserializer.deserialize_f64(visitor),
                },
                Schema::String => deserializer.deserialize_str(visitor),
                Schema::Object(_) => deserializer.deserialize_map(visitor),
                Schema::Array(_) => deserializer.deserialize_seq(visitor),
            }
        }
    }
    struct Visitor<'de> {
        schema: &'de Schema,
    }
    impl<'de> Visitor<'de> {
        pub fn new(schema: &'de Schema) -> Self {
            Self { schema }
        }
    }
    impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
        type Value = serde_json::Value;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            match &self.schema {
                Schema::String => formatter.write_str("a string"),
                Schema::Boolean => formatter.write_str("a boolean"),
                Schema::Number(n) => match n {
                    crate::schema::N::I64 => formatter.write_str("a i64"),
                    crate::schema::N::U64 => formatter.write_str("a u64"),
                    crate::schema::N::F64 => formatter.write_str("a f64"),
                },
                Schema::Object(_) => formatter.write_str("an object"),
                Schema::Array(_) => formatter.write_str("an array"),
            }
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
            Ok(serde_json::Value::String(value.to_owned()))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(serde_json::Value::String(v))
        }
        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(serde_json::Value::Bool(v))
        }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(v).unwrap(),
            ))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(serde_json::Value::Number(serde_json::Number::from(v)))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(serde_json::Value::Number(serde_json::Number::from(v)))
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            if let Schema::Object(fields) = self.schema {
                let mut object = serde_json::Map::new();
                while let Ok(Some(key)) = map.next_key::<&str>() {
                    if let Some(value_schema) = fields.get(key) {
                        match map.next_value_seed(Deserialize::new(value_schema)) {
                            Ok(value) => object.insert(key.to_owned(), value),
                            Err(err) => return Err(err),
                        };
                    } else {
                        match map.next_value_seed(Ignore) {
                            Ok(_) => {}
                            Err(err) => return Err(err),
                        }
                    }
                }
                Ok(serde_json::Value::Object(object))
            } else {
                Err(de::Error::custom("expected object"))
            }
        }
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            if let Schema::Array(item) = self.schema {
                let mut array = Vec::with_capacity(seq.size_hint().unwrap_or(100));
                while let Ok(Some(value)) = seq.next_element_seed(Deserialize::new(item)) {
                    array.push(value);
                }
                Ok(serde_json::Value::Array(array))
            } else {
                Err(de::Error::custom("expected array"))
            }
        }
    }
}
mod ignore {
    use serde::de;
    pub struct Ignore;
    impl<'de> de::Visitor<'de> for Ignore {
        type Value = Ignore;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("anything at all")
        }
        fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_f32<E>(self, _: f32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_char<E>(self, _: char) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_str<E>(self, _: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_borrowed_str<E>(self, _: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_string<E>(self, _: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_bytes<E>(self, _: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_borrowed_bytes<E>(self, _: &'de [u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_byte_buf<E>(self, _: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_some<D>(self, _: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            Ok(Ignore)
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Ignore)
        }
        fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            Ok(Ignore)
        }
        fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Ok(Ignore)
        }
        fn visit_map<A>(self, _: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            Ok(Ignore)
        }
        fn visit_enum<A>(self, _: A) -> Result<Self::Value, A::Error>
        where
            A: de::EnumAccess<'de>,
        {
            Ok(Ignore)
        }
    }
    impl<'de> de::DeserializeSeed<'de> for Ignore {
        type Value = Ignore;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_ignored_any(Ignore)
        }
    }
}
mod schema {
    use crate::de::Deserialize;
    use serde::de::DeserializeSeed;
    use serde_json::de::StrRead;
    use std::collections::HashMap;
    pub enum Schema {
        String,
        Number(N),
        Boolean,
        Object(HashMap<String, Box<Schema>>),
        Array(Box<Schema>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Schema {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Schema::String => ::core::fmt::Formatter::write_str(f, "String"),
                Schema::Number(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Number", &__self_0)
                }
                Schema::Boolean => ::core::fmt::Formatter::write_str(f, "Boolean"),
                Schema::Object(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Object", &__self_0)
                }
                Schema::Array(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Array", &__self_0)
                }
            }
        }
    }
    pub enum N {
        I64,
        U64,
        F64,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for N {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    N::I64 => "I64",
                    N::U64 => "U64",
                    N::F64 => "F64",
                },
            )
        }
    }
    impl Schema {
        pub fn from_str(&self, input: &str) -> serde_json::Result<serde_json::Value> {
            let mut deserializer = serde_json::Deserializer::new(StrRead::new(input));
            Deserialize::new(self).deserialize(&mut deserializer)
        }
        pub fn array(item: Schema) -> Schema {
            Schema::Array(Box::new(item))
        }
        pub fn i64() -> Schema {
            Schema::Number(N::I64)
        }
        pub fn u64() -> Schema {
            Schema::Number(N::U64)
        }
        pub fn f64() -> Schema {
            Schema::Number(N::F64)
        }
        pub fn object(map: HashMap<String, Schema>) -> Schema {
            Schema::Object(map.into_iter().map(|(k, v)| (k, Box::new(v))).collect())
        }
    }
}
pub use schema::Schema;
use serde::Deserialize;
struct Post {
    user_id: u64,
    id: u64,
    title: String,
    body: String,
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Post {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            #[doc(hidden)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            #[doc(hidden)]
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        1u64 => _serde::__private::Ok(__Field::__field1),
                        2u64 => _serde::__private::Ok(__Field::__field2),
                        3u64 => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "user_id" => _serde::__private::Ok(__Field::__field0),
                        "id" => _serde::__private::Ok(__Field::__field1),
                        "title" => _serde::__private::Ok(__Field::__field2),
                        "body" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"user_id" => _serde::__private::Ok(__Field::__field0),
                        b"id" => _serde::__private::Ok(__Field::__field1),
                        b"title" => _serde::__private::Ok(__Field::__field2),
                        b"body" => _serde::__private::Ok(__Field::__field3),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<Post>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Post;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct Post")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"struct Post with 4 elements",
                            ));
                        }
                    };
                    let __field1 = match _serde::de::SeqAccess::next_element::<u64>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                1usize,
                                &"struct Post with 4 elements",
                            ));
                        }
                    };
                    let __field2 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                2usize,
                                &"struct Post with 4 elements",
                            ));
                        }
                    };
                    let __field3 = match _serde::de::SeqAccess::next_element::<String>(&mut __seq)?
                    {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                3usize,
                                &"struct Post with 4 elements",
                            ));
                        }
                    };
                    _serde::__private::Ok(Post {
                        user_id: __field0,
                        id: __field1,
                        title: __field2,
                        body: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<u64> = _serde::__private::None;
                    let mut __field1: _serde::__private::Option<u64> = _serde::__private::None;
                    let mut __field2: _serde::__private::Option<String> = _serde::__private::None;
                    let mut __field3: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        _serde::de::MapAccess::next_key::<__Field>(&mut __map)?
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "user_id",
                                        ),
                                    );
                                }
                                __field0 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        u64,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field1 => {
                                if _serde::__private::Option::is_some(&__field1) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("id"),
                                    );
                                }
                                __field1 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        u64,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field2 => {
                                if _serde::__private::Option::is_some(&__field2) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("title"),
                                    );
                                }
                                __field2 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            __Field::__field3 => {
                                if _serde::__private::Option::is_some(&__field3) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("body"),
                                    );
                                }
                                __field3 =
                                    _serde::__private::Some(_serde::de::MapAccess::next_value::<
                                        String,
                                    >(
                                        &mut __map
                                    )?);
                            }
                            _ => {
                                let _ = _serde::de::MapAccess::next_value::<_serde::de::IgnoredAny>(
                                    &mut __map,
                                )?;
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => _serde::__private::de::missing_field("user_id")?,
                    };
                    let __field1 = match __field1 {
                        _serde::__private::Some(__field1) => __field1,
                        _serde::__private::None => _serde::__private::de::missing_field("id")?,
                    };
                    let __field2 = match __field2 {
                        _serde::__private::Some(__field2) => __field2,
                        _serde::__private::None => _serde::__private::de::missing_field("title")?,
                    };
                    let __field3 = match __field3 {
                        _serde::__private::Some(__field3) => __field3,
                        _serde::__private::None => _serde::__private::de::missing_field("body")?,
                    };
                    _serde::__private::Ok(Post {
                        user_id: __field0,
                        id: __field1,
                        title: __field2,
                        body: __field3,
                    })
                }
            }
            #[doc(hidden)]
            const FIELDS: &'static [&'static str] = &["user_id", "id", "title", "body"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Post",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<Post>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
