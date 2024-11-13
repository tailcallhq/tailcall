---
identity: true
---

# test-ref-other

```graphql @config
schema {
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
