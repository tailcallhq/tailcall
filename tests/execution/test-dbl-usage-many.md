---
expect_validation_error: true
---

# test-dbl-usage-many

```graphql @server
schema {
  query: Query
}

input Post {
  id: ID!
  title: String!
}

input User {
  id: ID!
  name: String!
}

type Query {
  post(input: Post!): Post @http(baseURL: "http://localhost:8080", path: "/user/{{.args.input.id}}")
  user(input: User!): User @http(baseURL: "http://localhost:8080", path: "/user/{{.args.input.id}}")
}
```
