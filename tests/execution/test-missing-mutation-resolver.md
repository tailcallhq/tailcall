---
error: true
---

# test-missing-mutation-resolver

```graphql @schema
schema {
  query: Query
  mutation: Mutation
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/user/1")
}

type User {
  id: ID
}

type UserInput {
  id: ID
}

type Mutation {
  createUser(input: UserInput!): User
}
```
