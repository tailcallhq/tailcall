---
expect_validation_error: true
---

# test-missing-mutation-resolver

```graphql @server
schema {
  query: Query
  mutation: Mutation
}

input UserInput {
  id: ID
}

type Mutation {
  createUser(input: UserInput!): User
}

type Query {
  user: User @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/user/1")
}

type User {
  id: ID
}
```
