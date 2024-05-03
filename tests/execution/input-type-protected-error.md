---
expect_validation_error: true
---

# input-type-protected-error

```graphql @server
schema {
  query: Query
  mutation: Mutation
}

input Input @protected {
  value: String
}

input NewPost {
  content: String@protected 
}

type Mutation {
  data(input: Input): String @expr(body: "value")
  newPost(post: NewPost): Post @http(baseURL: "", body: "{{.args.post}}", method: "POST", path: "/posts")
}

type Post {
  body: String!
  id: Int!
  title: String!
  userId: Int!
}

type Query {
  data: String @expr(body: "value")
}
```
