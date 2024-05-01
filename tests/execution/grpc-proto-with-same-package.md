# Grpc when multiple proto files have the same package name

```protobuf @file:foo.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package test;

message Foo {
  string foo = 1;
}

service FooService {
  rpc GetFoo (google.protobuf.Empty) returns (Foo) {}
}
```

```protobuf @file:bar.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package test;


message Bar {
  string bar = 1;
}

service BarService {
  rpc GetBar (google.protobuf.Empty) returns (Bar) {}
}
```

```graphql @server
schema
  @server(port: 8000, graphiql: true)
  @upstream(baseURL: "http://localhost:50051")
  @link(src: "foo.proto", type: Protobuf)
  @link(src: "bar.proto", type: Protobuf) {
  query: Query
}

type Query {
  foo: Foo! @grpc(method: "test.FooService.GetFoo")
  bar: Bar! @grpc(method: "test.BarService.GetBar")
}

type Foo {
  foo: String
}

type Bar {
  bar: String
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/test.FooService/GetFoo
  response:
    status: 200
    textBody: \0\0\0\0\n\n\x08test-foo

- request:
    method: POST
    url: http://localhost:50051/test.BarService/GetBar
  response:
    status: 200
    textBody: \0\0\0\0\n\n\x08test-bar
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { foo { foo } bar { bar } }
```
