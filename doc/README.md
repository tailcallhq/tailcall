### Adding GRPC Support to Tailcall Overview

This proposal outlines an approach for adding GRPC client support to Tailcall to allow calling GRPC services from Tailcall resolvers.


## Design Goals

The primary design goals are:

- Call GRPC services from Tailcall resolvers
- Generate GRPC client stubs from GraphQL schema
- Map between GraphQL and GRPC requests/responses
- No changes needed to Tailcall server


## Proposed Solution

The proposed solution is to:

Add a grpc section to the Tailcall config file, similar to http to configure GRPC endpoints:

```bash
grpc
  endpoints = [
    {port = 50051, proto="user.proto"},
    {port = 50052, proto="product.proto"}
  ]
```


Add a @grpc directive to mark GraphQL types as GRPC messages:

```bash
@grpc
type User {
  id: ID! 
  name: String!
}
```
- Generate GRPC proto files from GraphQL schemas with @grpc types.
- Use a GRPC client library like tonic to generate GRPC client stubs.
- Call the GRPC client stub from Tailcall resolvers, handling request/response mapping.


## Detailed Implementation using Tonic

Tonic is a gRPC over HTTP/2 implementation focused on high performance, interoperability, and flexibility. It's written in Rust, making it an excellent choice for implementing the gRPC feature in Tailcall. Here's a detailed section on how we can use Tonic:

1. Add Tonic and Prost to Your Dependencies: 

Tonic is the gRPC client and server implementation, and Prost is a Protocol Buffers implementation in Rust. Add them to your `Cargo.toml` file:

```bash
[dependencies]
tonic = "0.5"
prost = "0.9"
```

2. Generate gRPC Client Stubs

- Add @grpc directive to GraphQL schema to specify GRPC service and method for each field.
- Run a custom tool on startup to parse schema and generate .proto files for @grpc fields.
- The tool extracts the service, method, request and response types.
- It generates .proto files defining the RPC service and messages.

```bash
graphql
type User @grpc(service: "user.UserService") {
  name: String @grpc(method: "GetUserName")
  age: Int @grpc(method: "GetUserAge") 
}
```

Parse the schema and generate the corresponding .proto files. Tonic can then generate Rust code from the .proto files.

3. Handle Schema Conversion

- Create a new grpc module to handle gRPC code generation. This module would contain the logic to parse  the schema and generate .proto files.
- In the schema generation code, after creating the JSON schema, call the new grpc module to handle gRPC schema generation.
- The grpc module would parse the schema AST and look for types annotated with @grpc directives. It would extract the service name, method name, request and response types for each gRPC endpoint.
- Using this extracted information, the grpc module can generate a .proto file defining the gRPC service, messages, and RPC methods.
- The grpc module should write the generated .proto files to disk, in a grpc sub-directory alongside the JSON schemas.

For example, the GraphQL schema above would generate something like:
```bash
service UserService {
  rpc GetUserName(UserNameRequest) returns (UserNameResponse);
  rpc GetUserAge(UserAgeRequest) returns (UserAgeResponse);
}
```

4. Make gRPC Calls

- Use Tonic as before to generate Rust code from the .proto files. 
- This provides gRPC client stubs for each service.
- In the Tailcall resolver, construct a request from the GraphQL arguments.
- Call the appropriate stub method, passing the request.
- Await the gRPC response, then map it back to the GraphQL schema.

5. Map Requests and Responses

- Convert GraphQL request to gRPC request.
- Call stub and get gRPC response.
- Map gRPC response back to GraphQL.


## Technical Details

# Components affected:

- Config loading - parse grpc endpoints
- Schema generation - generate .proto files
- Resolvers - call GRPC client stubs

# Libraries needed:

tonic - for GRPC clients
prost - for generating GRPC stubs from .proto

We'll need serialization/deserialization between GraphQL and GRPC requests/responses.
The .proto files will be generated from GraphQL schema and stored alongside .graphql files or in a separate proto folder.

## Benefits

- Allows calling GRPC services easily
- No server changes needed
- Leverages code generation

## Drawbacks

- Overhead of request/response conversion
- Limitations mapping GRPC and GraphQL types

## Conclusion

This proposal outlines an approach to add GRPC client support with minimal changes by focusing on client stub generation and request/response mapping.