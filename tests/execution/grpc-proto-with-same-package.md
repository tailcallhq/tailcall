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

```yaml @config
server:
  port: 8000
links:
  - src: "foo.proto"
    type: Protobuf
  - src: "bar.proto"
    type: Protobuf
```

```graphql @schema
schema {
  query: Query
}

type Query {
  foo: Foo! @grpc(url: "http://localhost:50051", method: "test.FooService.GetFoo")
  bar: Bar! @grpc(url: "http://localhost:50051", method: "test.BarService.GetBar")
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
