# Adding GRPC Support to Tailcall

## Objective

The primary objective is to develop a clear and well-documented design and architecture that enables GRPC support in Tailcall. This proposal aims to create documentation that outlines the design and architecture, making it easy to understand, practical to implement, easy to maintain, and effective in addressing the problem at hand. It's important to note that this proposal focuses on documentation and does not include the actual implementation of the approach.


### Design Goals

The design goals for adding GRPC support to Tailcall are as follows:

1. **Coexistence**: Allow configuring GRPC servers alongside existing HTTP servers within the Tailcall configuration.

2. **Code Generation**: Generate GRPC server stubs from GraphQL schemas with minimal manual effort.

3. **Integration**: Integrate GRPC support cleanly with the existing Tailcall architecture to minimize disruption.

### Proposed Solution

The proposed solution includes the following key components:

1. **Tailcall Configuration**: Introduce a `grpc` section within the Tailcall configuration file, similar to the existing `http` section. This will allow users to specify GRPC-specific settings, such as the port on which the GRPC server should listen.

   ```bash
   grpc:
     port: 50051
  
  ```

 2. **Annotation Directive**: Create a `@grpc` directive that can be applied to GraphQL types to mark them as GRPC messages.

 ```bash
@grpc

type User {
  id: ID! 
  name: String!
}
```
3. **Proto File Generation**: Implement a process to generate GRPC .proto files from GraphQL schemas containing types marked with the @grpc directive.

4. **Code Generation**:  Utilize established GRPC code generation tools, such as grpc-rust, to generate GRPC server stubs from the generated .proto files. This automated approach ensures efficiency and consistency.

5. **Server Integration**:  Start a GRPC server alongside the HTTP server to handle incoming GRPC requests. This ensures that both GRPC and HTTP services coexist seamlessly.

6. **Data Conversion**:  Develop mechanisms for converting between GRPC requests and responses and GraphQL types. This allows the reuse of existing Tailcall resolvers.


### Implementation Details      

`@grpc` Directive: Specify how the `@grpc` directive marks a GraphQL object type as a GRPC message, and how this annotation is recognized and processed.

Proto File Generation: Describe the process of generating .proto files from the GraphQL schema and how the content of these files is structured.

GRPC Server: Discuss how the GRPC server listens on the configured port and handles the conversion of requests and responses between GRPC and GraphQL types.

Reuse of Tailcall Resolvers: Explain how existing Tailcall resolvers are adapted for handling the conversion between GRPC and GraphQL types.


## Benefits

The proposed solution offers several advantages:

Seamless Integration: Enables the addition of GRPC support without requiring major architectural changes to the Tailcall project.

Code Reuse: Leverages existing schema and resolver code, allowing for efficient development and maintenance.

Hybrid Services: Provides an easy way to add GRPC APIs alongside existing GraphQL services within Tailcall.


## Drawbacks

There are some potential drawbacks to consider:

Increased Complexity: Introducing a GRPC server alongside the HTTP server adds complexity to the system.

Conversion Overhead: The conversion process between GRPC and GraphQL types may introduce some overhead.

Proto Files: The need to generate and maintain .proto files adds an extra layer of complexity.

## Conclusion

This proposal outlines a comprehensive and well-documented approach for adding GRPC support to Tailcall. By integrating with existing tools, reusing schema and resolver code, and keeping changes localized, this proposal aims to address the GRPC support requirement effectively. Feedback and suggestions from the Tailcall community and maintainers are welcome to enhance the proposal further.# Adding GRPC Support to Tailcall

