---
error: true
---

# test-graphqlsource-no-base-url

```graphql @schema
schema {
  query: Query
}

type Post {
  id: Int!
  user: User @graphQL(name: "user", args: [{key: "id", value: "{{.value.userId}}"}])
}

type Query {
  post(id: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.args.id}}")
}

type User {
  id: Int
  name: String
}
```
