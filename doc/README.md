## Adding GRPC Support to Tailcall Overview

This proposal outlines an approach for adding GRPC support to Tailcall to allow generating GRPC server stubs from GraphQL schemas.


# Design Goals

The primary design goals are:

Allow configuring GRPC servers alongside HTTP servers in Tailcall config
Generate GRPC server stubs from GraphQL schemas
Enable handling GRPC requests with Tailcall resolvers
Integrate cleanly with existing Tailcall architecture
Proposed Solution
The proposed solution is to:

Add a grpc section to the Tailcall config file, similar to http:

```bash
grpc:
  port: 50051
```


Add a @grpc directive to mark GraphQL types as GRPC messages:

```bash
@grpc

type User {
  id: ID! 
  name: String!
}
```


Generate GRPC proto files from GraphQL schemas with @grpc types.
Use a GRPC code generation tool like grpc-rust to generate GRPC server stubs from the proto files.
Start a GRPC server alongside the HTTP server to handle requests.
Convert between GRPC requests/responses and GraphQL types to reuse existing Tailcall resolvers.

Implementation Details

The @grpc directive would mark a GraphQL object type as a GRPC message.
The proto file generator would output .proto files from the GraphQL schema, containing any @grpc types as messages.
The GRPC server would listen on the configured port and handle request/response conversion.
Existing Tailcall resolvers could be reused by converting between GRPC and GraphQL types.

## Benefits

Allows supporting GRPC without major architecture changes.
Leverages existing schema and resolver code.
Provides easy way to add GRPC APIs alongside GraphQL.

## Drawbacks

Additional complexity of running a GRPC server.
Overhead of request/response conversion.
Limitations converting between GraphQL and protobuf types.
Alternatives Considered

Code generation for GRPC server stubs from GraphQL schemas. This would require more custom tooling vs leveraging existing GRPC tools.
Implementing GRPC support directly in Tailcall. This would require significant changes to core server architecture.

## Conclusion

This proposal outlines a way to add GRPC support by integrating with existing tools, reusing schema and resolver code, and keeping changes localized. Feedback welcome!
