# Tailcall fixtures

Package that contains configs and fixtures used in tests and examples to be shared among different parts of tailcall plus helper functions to resolve these files in tests.

## gRPC binary files

Instructions to generate those files:

1. go to `src/core/proto_reader/fetch.rs`
2. modify `GrpcReflection.execute` function to print the request `body` as string and response `body` as base64 encoded
3. convert the base64 into bin files using online base64 to file websites or manually
4. rename the files and replace the old ones
