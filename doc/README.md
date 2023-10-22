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