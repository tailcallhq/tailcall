# test-dbl-usage

```graphql @schema
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
  user(input: User!): User @http(url: "http://localhost:8080/user/{{.args.input.id}}")
  post(input: Post!): Post @http(url: "http://localhost:8080/user/{{.args.input.id}}")
}
```
