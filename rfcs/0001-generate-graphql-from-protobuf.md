# Generate graphQL config from protobuf definitions

Generate Tailcall config from protobuf definitions passed to CLI.

## Conversion from protobuf

Next assumptions are made while implementing config generation:

- proto3 standard only
- all of the service's methods are added as query, mutations are ignored since it's hard to distinct those from service definition

Useful links that can help with implementation:

- [Protobuf spec](https://protobuf.dev/programming-guides/proto3/)
- [graphql-mesh conversion from GRPC to graphQL](https://github.com/ardatan/graphql-mesh/tree/master/packages/legacy/handlers/grpc) - repo with implementation in typescript and test suites for different input protobuf files that Tailcall can borrow to test its implementation
- [Apollo schema-driven-grpc](https://github.com/apollosolutions/directive-driven-grpc) - experimental implementation for grpc including conversion from protobuf to graphQL and grpc directive implementation

### Scalar types

- [list of available scalar types in protobuf](https://protobuf.dev/programming-guides/proto3/#scalar)
- [list of builtin scalars in graphQL](https://graphql.org/learn/schema/#scalar-types)
- [example of conversion](https://github.com/ardatan/graphql-mesh/blob/master/packages/legacy/handlers/grpc/test/__snapshots__/handler.spec.ts.snap#L933-L960)

Protobuf has more builtin scalar types than graphQL and because of this not every protobuf scalar type could be natively mapped to graphQL type and some of them will require implementing additional scalar types in Tailcall.

Will require to implement scalars for:

- BigInt to represent integers more than 32 bit length
- UnsignedInt to represent unsigned integers
- Bytes to represent list of bytes
- Empty to represent empty message

How to convert:

- implement additional required scalar types
- when converting from protobuf use graphQL builtin or added scalar types

### Message Type

- [protobuf Message Type](https://protobuf.dev/programming-guides/proto3/#simple)
- [graphQL Object Type](https://graphql.org/learn/schema/#object-types-and-fields)
- [graphQL Input Type](https://graphql.org/learn/schema/#input-types)

Message Type basically represents complex type that consist of fields. To represent this in graphql we have two available options: `type` and `input`. Exact used representation should be picked based on usage.

How to convert:

- to prevent name clashes the name of generated type should be `<package_name>__<...parent_message>__<message_name>` where `<package_name>` is package of the message with `.` replaced with `__` and `<parent_message>` is the names of parent messages for this messages if any separated by `__`
- if message doesn't contain fields replace it with scalar type `Empty`
- when message is used as output type in method than add new `type` in graphql with the name defined above
- when message is used as argument in method than add new `input` in graphql with the name with additional postfix `__Input`

### Message fields

Fields are the parts of Message Type and should be converted to fields of the output type.

How to convert:

- the name of the field should be converted to `camelCase` according to graphQL style guide and the way protobuf is converted from/to JSON
- since protobuf allows to omit values in most cases all of the generated fields should be nullable except for only of cases for scalars of output types since they would be populated on parsing from grpc in any case
- `optional` doesn't affect output because of above
- `repeated` should be converted as list
- `map` see [Maps](#maps)

### Enum

- [protobuf Enum](https://protobuf.dev/programming-guides/proto3/#enum)
- [graphQL Enum](https://graphql.org/learn/schema/#enumeration-types)

Enum in protobuf mostly maps to enums in graphQL with only difference in the way how the value is stored in serialized data. Since Tailcall uses json conversion for protobuf with settings that stored the names of variants and also that how async_graphql works we may ignore number values associated with variants and use just variant names.

How to convert:

- for the name of enum in graphQL use similar approach as for [message type](#message-type)
- for variants preserve defined names in protobuf

### Service

- [protobuf Service](https://protobuf.dev/programming-guides/proto3/#services)

Services found in protobuf definition represents available remote operation. That should be put into root Query type.

How to convert:

- the name of the field should be `<package_name>__<service_name>` lowercased and for `<package_name>` dots replaced with `__`
- input argument if it's not `google.protobuf.Empty` should be added as non-nullable argument to field with name `input` and corresponding input type should be added to schema, including all nested types
- output type should be added as non-nullable output type of the field and corresponding output type should be added to schema, including all nested types
- directive `@grpc` should be added to the field with the path to the service as `method` and with `body: "{{args.input}}"` if argument is specified

### Google types

Some predefined external types like with prefix `google.` could be imported and used in protobuf. Corresponding types should be read and added to output schema as usual types.

How to convert:

- when google types are found load its definition as usual and add input or output type depending on its usage
- for `google.protobuf.Empty` use scalar `Empty` to define null output for methods that return nothing

### Maps

- [protobuf Maps](https://protobuf.dev/programming-guides/proto3/#maps)

Maps in protobuf is basically syntax sugar to new message type that contains list of key-value pairs. For details see [encoding doc](https://protobuf.dev/programming-guides/encoding/#maps). So Tailcall should create new additional type for that and use it as type of respective field

TODO: check that changing maps to message type as showed in encoding docs is works as expected for current protobuf conversion in Tailcall

### Oneof

- [protobuf Oneof](https://protobuf.dev/programming-guides/proto3/#oneof)
- [graphQL oneof rfc](https://github.com/graphql/graphql-spec/pull/825)
- [graphQL union type](https://graphql.org/learn/schema/#union-types)

Oneof feature of protobuf doesn't have exact mapping to graphql due to some restriction. We may consider not support it for a while or provide partial support.

How to convert:

- oneof should be represented by new type with the name definition like for [Message Type](#message-type) with postfix `__Oneof`
- for input types use `@oneof` directive or option available in async_graphql. It's supports well scalar and compound types
- for output types use Union type, but it's limited to only compound types though protobuf's oneof could use scalars as well

### Comments

Comments for protobuf definitions should be preserved whenever possible in graphQL schema

## Example

Consider that protobuf file as an input:

```protobuf
syntax = "proto3";

package io.xtech;

import "google/protobuf/timestamp.proto";

enum Genre {
    UNSPECIFIED = 0;
    ACTION = 1;
    DRAMA = 2;
}

/**
 * movie message payload
 */
message Movie {
    string name = 1;
    int32 year = 2;
    float rating = 3;

    /**
     * list of cast
     */
    repeated string cast = 4;
    google.protobuf.Timestamp time = 5;
    Genre genre = 6;
}

message EmptyRequest {}

message movieRequest {
    Movie movie = 1;
}

message movie_request_by_ids {
    repeated string movieIds = 1;
}

/**
 * movie result message, contains list of movies
 */
message MoviesResult {
    /**
     * list of movies
     */
    repeated Movie result = 1;
}

service Example {
  /**
  * get all movies
  */
  rpc GetMovies (movieRequest) returns (MoviesResult) {}

  /**
  * get movies
  */
  rpc RetrieveMovies (movie_request_by_ids) returns (MoviesResult) {}
}
```

Expected output for this file would be:

```graphql
schema {
  query: Query
}

type Query {
  """get all movies"""
  io_xtech_Example_GetMovies(input: io__xtech__movie_request_Input): io__xtech__MoviesResult @grpcMethod(rootJsonName: "Root0", objPath: "io.xtech.Example", methodName: "GetMovies", responseStream: false)
  """get movies"""
  io_xtech_Example_RetrieveMovies(input: io__xtech__movie_request_by_ids_Input): io__xtech__MoviesResult @grpcMethod(rootJsonName: "Root0", objPath: "io.xtech.Example", methodName: "RetrieveMovies", responseStream: false)
}

"""movie result message, contains list of movies"""
type io__xtech__MoviesResult {
  """list of movies"""
  result: [io__xtech__Movie]
}

"""movie message payload"""
type io__xtech__Movie {
  name: String
  year: Int
  rating: Float
  """list of cast"""
  cast: [String]
  time: google__protobuf__Timestamp
  genre: io__xtech__Genre
}

type google__protobuf__Timestamp {
  seconds: BigInt
  nanos: Int
}

enum io__xtech__Genre {
  UNSPECIFIED
  ACTION
  DRAMA
}

input io__xtech__movieRequestInput {
  movie: io__xtech__MovieInput
}

"""movie message payload"""
input io__xtech__MovieInput {
  name: String
  year: Int
  rating: Float
  """list of cast"""
  cast: [String]
  time: google__protobuf__TimestampInput
  genre: io__xtech__Genre
}

input google__protobuf__TimestampInput {
  seconds: BigInt
  nanos: Int
}

input io__xtech__movie_request_by_idsInput {
  movieIds: [String]
}
```
