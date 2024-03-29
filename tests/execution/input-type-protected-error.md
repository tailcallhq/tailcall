---
expect_validation_error: true
---

##### only

# input-type-protected-error

```graphql @server
schema {
  query: Query
  mutation: Mutation
}

type Query {
    data: String @const(data: "value")
}

type Mutation {
    data(input: Input): String @const(data: "value")
}

input Input @protected {
  name: String
}
```