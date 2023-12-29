---
title: "@grpc"
---

The **@grpc** operator is an essential GraphQL custom directive designed to interface with gRPC services. It allows GraphQL queries to directly invoke gRPC services, thereby bridging two powerful technologies. This directive is particularly useful when integrating GraphQL with microservices that expose gRPC endpoints.

### Using the `@grpc` Operator

The **@grpc** operator allows GraphQL fields to be resolved using gRPC services. Here's an example demonstrating its usage in a GraphQL schema:

```graphql showLineNumbers
type Query {
  users: [User] @grpc(service: "UserService", method: "ListUsers", protoPath: "./proto/user_service.proto")
}
```

In this example, when the `users` field is queried, the GraphQL server will make a gRPC request to the `ListUsers` method of the `UserService`.

### Sample proto File

The `.proto` file defines the structure of the gRPC service and its methods. Here is a simplified example:

```proto showLineNumbers
syntax = "proto3";

service UserService {
  rpc ListUsers (UserListRequest) returns (UserListReply) {}
  rpc GetUser (UserGetRequest) returns (UserGetReply) {}
}

message UserListRequest {
  // Request parameters
}

message UserListReply {
  // Reply structure
}

message UserGetRequest {
  // Reply structure
}

message UserGetReply {
  // Reply structure
}
```

### service

Indicates the gRPC service to be called. This should match the service name as defined in the `.proto` file.

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      # highlight-start
      service: "UserService"
      # highlight-end
      method: "ListUsers"
      protoPath: "./proto/user_service.proto"
    )
}
```

### method

Indicates the specific gRPC method to be invoked within the specified service. This should match the method name as defined in the `.proto` file.

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      service: "UserService"
      # highlight-start
      method: "ListUsers"
      # highlight-end
      protoPath: "./proto/user_service.proto"
    )
}
```

### protoPath

Path to the `.proto` file, containing service and method definitions for request/response encoding and decoding. The path can be relative or absolute. If the path is relative, it is resolved relative to the directory where the tailcall command is run from.

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      service: "UserService"
      method: "ListUsers"
      # highlight-start
      protoPath: "./proto/user_service.proto"
      # highlight-end
    )
}
```

### baseURL

Indicates the base URL for the gRPC API. If omitted, the default URL defined in the `@upstream` operator is used.

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      service: "UserService"
      method: "ListUsers"
      protoPath: "./proto/user_service.proto"
      # highlight-start
      baseURL: "https://grpc-server.example.com"
      # highlight-end
    )
}
```

### body

Outlines the arguments for the gRPC call. The `body` field is used to specify the arguments for the gRPC call. It can be static or dynamic. Here's an example:

```graphql showLineNumbers
type UserInput {
  id: ID
}

type Query {
  user(id: UserInput!): User
    @grpc(
      service: "UserService"
      method: "GetUser"
      protoPath: "./proto/user_service.proto"
      # highlight-start
      body: "{{args.id}}"
      # highlight-end
    )
}
```

### headers

Custom headers for the gRPC request can be specified using the `headers` argument. This is particularly useful for passing authentication tokens or other contextual information.

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      service: "UserService"
      method: "ListUsers"
      protoPath: "./proto/user_service.proto"
      baseURL: "https://grpc-server.example.com"
      # highlight-start
      headers: [{key: "X-CUSTOM-HEADER", value: "custom-value"}]
      # highlight-end
    )
}
```

### groupBy

The `groupBy` argument is used to optimize batch requests by grouping them based on specified response keys. This can significantly improve performance in scenarios with multiple, similar requests.

For using the groupBy capability, the response type of the gRPC method should be a list of objects. For example, if the response type of the gRPC method is `UserListReply`, then the `groupBy` argument can be used as follows:

```graphql showLineNumbers
type Query {
  users(id: UserInput!): User
    @grpc(
      service: "UserService"
      method: "ListUsers"
      protoPath: "./proto/user_service.proto"
      baseURL: "https://grpc-server.example.com"
      # highlight-start
      groupBy: ["id"]
      # highlight-end
    )
}
```

The **@grpc** operator is a powerful tool for GraphQL developers, allowing for seamless integration with gRPC services. By understanding and utilizing its various fields, developers can create efficient, streamlined APIs that leverage the strengths of both GraphQL and gRPC.
