---
skip: true
---

# Grpc oneof types

TODO: Skipped because fixing this test will take time

```protobuf @file:oneof.proto
syntax = "proto3";

package oneof;

message Payload {
	string payload = 1;
}

message Command {
	string command = 1;
}

message Request {
	string usual = 1;

  oneof req_oneof {
		Payload payload = 2;
		Command command = 3;
	}
}

message Response {
	int32 usual = 1;

  oneof resp_oneof {
		Payload payload = 2;
		Command command = 3;
		string response = 4;
	}
}

service OneOfService {
  rpc GetOneOf (Request) returns (Response) {}
}

```

```yaml @config
server:
  port: 8000
upstream:
  httpCache: 42
  batch:
    delay: 10
links:
  - src: "oneof.proto"
    type: Protobuf
```

```graphql @schema
schema {
  query: Query
}

input oneof__CommandInput {
  command: String
}

input oneof__PayloadInput {
  payload: String
}

input oneof__Request__Var0__Var {
  payload: oneof__PayloadInput!
  usual: String
}

input oneof__Request__Var0__Var0 {
  flag: Boolean!
  payload: oneof__PayloadInput!
  usual: String
}

input oneof__Request__Var0__Var1 {
  optPayload: oneof__PayloadInput!
  payload: oneof__PayloadInput!
  usual: String
}

input oneof__Request__Var1__Var {
  command: oneof__CommandInput!
  usual: String
}

input oneof__Request__Var1__Var0 {
  command: oneof__CommandInput!
  flag: Boolean!
  usual: String
}

input oneof__Request__Var1__Var1 {
  command: oneof__CommandInput!
  optPayload: oneof__PayloadInput!
  usual: String
}

input oneof__Request__Var__Var {
  usual: String
}

input oneof__Request__Var__Var0 {
  flag: Boolean!
  usual: String
}

input oneof__Request__Var__Var1 {
  optPayload: oneof__PayloadInput!
  usual: String
}

interface oneof__Response__Interface {
  usual: Int
}

union oneof__Response = oneof__Response__Var | oneof__Response__Var0 | oneof__Response__Var1 | oneof__Response__Var2

type Query {
  oneof__OneOfService__GetOneOfVar0(request: oneof__Request__Var0__Var!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar1(request: oneof__Request__Var0__Var0!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar2(request: oneof__Request__Var0__Var1!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar3(request: oneof__Request__Var1__Var!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar4(request: oneof__Request__Var1__Var0!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar5(request: oneof__Request__Var1__Var1!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar6(request: oneof__Request__Var__Var!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar7(request: oneof__Request__Var__Var0!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
  oneof__OneOfService__GetOneOfVar8(request: oneof__Request__Var__Var1!): oneof__Response!
    @grpc(url: "http://localhost:50051", body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
}

type oneof__Command {
  command: String
}

type oneof__Payload {
  payload: String
}

type oneof__Response__Var implements oneof__Response__Interface {
  usual: Int
}

type oneof__Response__Var0 implements oneof__Response__Interface {
  payload: oneof__Payload!
  usual: Int
}

type oneof__Response__Var1 implements oneof__Response__Interface {
  command: oneof__Command!
  usual: Int
}

type oneof__Response__Var2 implements oneof__Response__Interface {
  response: String!
  usual: Int
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/oneof.OneOfService/GetOneOf
  response:
    status: 200
    textBody: \0\0\0\0\x09\x08\x05\x1A\x05\n\x03end
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        oneof__OneOfService__GetOneOfVar3(request: { command: { command: "start" } }) {
          ... on oneof__Response__Interface {
            usual
          }
          ... on oneof__Response__Var1 {
            command {
              command
            }
          }
        }
      }
```
