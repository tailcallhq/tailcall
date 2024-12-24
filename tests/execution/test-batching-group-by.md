---
identity: true
---

# test-batching-group-by

```yaml @config
server:
  port: 4000
upstream:
  batch:
    delay: 1
    maxSize: 1000
```

```graphql @schema
schema @server @upstream {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @http(url: "http://abc.com/users", batchKey: ["id"], query: [{key: "id", value: "{{.value.userId}}"}])
  userId: Int!
}

type Query {
  posts: [Post] @http(url: "http://abc.com/posts?id=1&id=11")
}

type User {
  id: Int
  name: String
}
```
