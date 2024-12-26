---
identity: true
---

# test-ref-other

```yaml @config
server:
  port: 8000
```

```graphql @schema
schema @server(port: 8000) @upstream {
  query: Query
}

type InPost {
  get: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
}

type Post {
  id: Int!
  userId: Int!
}

type Query {
  posts: InPost
}
```
