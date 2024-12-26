---
error: true
---

# input-type-protected-error

```graphql @schema
schema {
  query: Query
  mutation: Mutation
}

type Query {
  data: String @expr(body: "value")
}

type Mutation {
  data(input: Input): String @expr(body: "value")
  newPost(post: NewPost): Post @http(url: "/posts", method: POST, body: "{{.args.post}}")
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
