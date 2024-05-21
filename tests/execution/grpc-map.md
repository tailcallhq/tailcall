# Grpc map type

```protobuf @file:map.proto
syntax = "proto3";

package map;

message MapRequest {
    map<string, string> map = 1;
}

message MapResponse {
    map<int32, string> map = 1;
}

service MapService {
  rpc GetMap (MapRequest) returns (MapResponse) {}
}

```

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://localhost:50051", httpCache: true, batch: {delay: 10})
  @link(src: "map.proto", type: Protobuf) {
  query: Query
}

schema @server @upstream {
  query: Query
}

input map__MapRequest @tag(id: "map.MapRequest") {
  map: JSON!
}

type Query {
  map__MapService__GetMap(mapRequest: map__MapRequest!): map__MapResponse!
    @grpc(body: "{{.args.mapRequest}}", method: "map.MapService.GetMap")
}

type map__MapResponse @tag(id: "map.MapResponse") {
  map: JSON!
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/map.MapService/GetMap
  response:
    status: 200
    textBody: \0\0\0\0\x12\n\t\x08\x01\x12\x05value
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        map__MapService__GetMap(mapRequest: { map: { key: "value" } }) {
          map
        }
      }
```
