# Module Documentation: `tailcall.runtime.model`

The `tailcall.runtime.model` module provides an intermediate representation (IR) for a GraphQL document, which can be
serialized and used to resolve fields into actual values.

The central data structure of this module is the `Blueprint` case class, which contains a list of `Definition`s.
A `Definition` can be any of the following:

- `ObjectTypeDefinition`: defines an object type in the GraphQL schema, with a name, a list of `FieldDefinition`s, and
  an optional description.
- `InputObjectTypeDefinition`: defines an input object type in the GraphQL schema, with a name, a list
  of `InputFieldDefinition`s, and an optional description.
- `SchemaDefinition`: defines the overall schema for the GraphQL document, with optional query, mutation, and
  subscription types, a list of `Directive`s, and an optional description.
- `ScalarTypeDefinition`: defines a scalar type in the GraphQL schema, with a name, a list of `Directive`s, and an
  optional description.

`FieldDefinition` and `InputFieldDefinition` define a field for an object type or an input object type, respectively.
They have a name, a list of arguments (if any), a return type (specified using the `Type` trait), a resolver function (
optional for `FieldDefinition`s), a list of `Directive`s, and an optional description.

`Directive` defines a directive in the GraphQL schema, with a name, a map of arguments, and an index (used for sorting).

`Type` is a trait that can be either a `NamedType` (with a name and a flag indicating whether it's non-null) or
a `ListType` (with a type and a flag indicating whether it's non-null). It has methods for rendering the type as a
string and for changing the name of the type.

The `Blueprint` case class has several methods, including:

- `digest`: generates a `Digest` from the `Blueprint`.
- `toGraphQL`: generates a GraphQL schema using `GraphQLGenerator` and a `HttpDataLoader`.
- `schema`: returns the `SchemaDefinition` from the `Blueprint`, if present.
- `endpoints`: extracts a list of `Endpoint`s from the `FieldDefinition`s in the `ObjectTypeDefinition`s in
  the `Blueprint`.

Overall, this module provides a way to represent a GraphQL schema as a set of case classes, which can be manipulated
programmatically and used to generate a GraphQL schema or to resolve fields in a GraphQL query. The `Blueprint` contains
all the necessary information to generate a GraphQL endpoint, including the schema of the GraphQL service and a way to
resolve all the data.
