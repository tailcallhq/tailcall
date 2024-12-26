---
identity: true
---

# test-graphqlsource

```graphql @schema
schema @server @upstream {
  query: Query
}

type Post {
  id: Int!
  user: User
    @graphQL(args: [{key: "id", value: "{{.value.userId}}"}], url: "http://localhost:8000/graphql", name: "user")
  userId: Int!
}

type Query {
  post(id: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.args.id}}")
}

type User {
  id: Int
  name: String
}
```
