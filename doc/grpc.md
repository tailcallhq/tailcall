# Adding gRPC Support to Tailcall

This document provides an overview of the process for incorporating gRPC support into Tailcall.

## Problem Statement

gRPC, a protocol widely employed for microservices communication due to its high performance, schema-driven nature, and compatibility with multiple programming languages, offers a valuable addition to Tailcall.

## Implementation

To introduce gRPC support for Tailcall, we will introduce a `@gRPC` operator to the schema. This operator will indicate that queries for specific types should be resolved through gRPC calls to specified gRPC server. The arguments for the `@gRPC` operator will specify the method to be invoked.

However, a critical challenge in gRPC integration lies in the management of proto files, which are necessary for initializing the gRPC client and calling methods. Storing these files within the Tailcall proxy or any other location gives rise to several issues:

- Proto files must be accurately copied to the specified locations for each service.
- The proxy must handle the retrieval of multiple proto files.
- After upgrading the service, the proto file needs to be updated each time, introducing potential errors and tedious maintenance.
- Implementing support for Blue-Green deployments becomes complex.

### Solution

gRPC provides a reflection feature that enables a gRPC client to dynamically query information about the services, methods, and message types available on a gRPC server. This functionality can be harnessed for initializing the gRPC client and sending requests without the reliance on proto files.

gRPC reflection is supported in multiple programming languages such as 
- Go
- C++
- Rust
- Python
- Java

### Steps

1. When the Tailcall proxy encounters the `@gRPC` operator specified in the requested type:
   
   - The Tailcall proxy will initialize a gRPC reflection client that leverages gRPC reflection to fetch the request and response schemas from the specified gRPC server.

   - The reflection client will be implemented using the [tonic-reflection](https://github.com/hyperium/tonic/tree/master/tonic-reflection) library.

2. The Tailcall proxy will utilize the schema provided by the reflection client to initialize a gRPC client and invoke the method specified in the `@gRPC` operator argument.

   - The gRPC client will be implemented using the [tonic](https://github.com/hyperium/tonic) library.

3. The Tailcall proxy uses the gRPC response and maps it to the GraphQL query.

## Schema Example

```graphql
schema
  @server(port: 8000, enableGraphiql: "/graphiql", enableQueryValidation: false, hostname: "0.0.0.0")
  @upstream {
  query: Query
}

type Query {
  posts: [Post] @gRPC(method: "Blog.Posts.getPosts")
}


type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
}

```

## Further Implementation
- Add TLS support
- Add streaming support?

