# test-dbl-usage

```graphql @config
schema {
  query: Query
}

type User {
  id: ID!
  name: String!
}

input Post {
  id: ID!
  title: String!
}

type Query {
  user(input: User!): User @http(path: "/user/{{.args.input.id}}", baseURL: "http://localhost:8080")
  post(input: Post!): Post @http(path: "/user/{{.args.input.id}}", baseURL: "http://localhost:8080")
}
```
