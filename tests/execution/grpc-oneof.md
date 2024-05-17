# Grpc oneof types

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

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://localhost:50051", httpCache: true, batch: {delay: 10})
  @link(src: "oneof.proto", type: Protobuf) {
  query: Query
}

input oneof__CommandInput @tag(id: "oneof.Command") {
  command: String
}

input oneof__PayloadInput @tag(id: "oneof.Payload") {
  payload: String
}

input oneof__Request @tag(id: "oneof.Request") {
  command: oneof__CommandInput
  payload: oneof__PayloadInput
  usual: String
}

type Query {
  oneof__OneOfService__GetOneOf(request: oneof__Request!): oneof__Response!
    @grpc(body: "{{.args.request}}", method: "oneof.OneOfService.GetOneOf")
}

type oneof__Command @tag(id: "oneof.Command") {
  command: String
}

type oneof__Payload @tag(id: "oneof.Payload") {
  payload: String
}

type oneof__Response @tag(id: "oneof.Response") {
  command: oneof__Command
  payload: oneof__Payload
  response: String
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
        oneof__OneOfService__GetOneOf(request: { command: { command: "start" } }) {
          payload { payload }
          command { command }
          response
          usual
        }
      }
```
