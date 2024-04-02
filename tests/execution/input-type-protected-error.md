---
expect_validation_error: true
---

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
  newPost(post: NewPost): Post @http(baseURL: "", path: "/posts", method: POST, body: "{{args.post}}")
}

input Input @protected {
  value: String
}

input NewPost {
  content: String @protected
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
}
```
