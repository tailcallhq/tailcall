---
expect_validation_error: true
---

# test-dbl-usage

```graphql @server
schema {
  query: Query
}

input User {
  id: ID!
  name: String!
}

type Query {
  user(input: User!): User @http(baseURL: "http://localhost:8080", path: "/user/{{.args.input.id}}")
}
```
