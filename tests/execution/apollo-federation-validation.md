---
error: true
---

# Apollo federation validation

```yaml @config
server:
  port: 8000
  enableFederation: true
upstream:
  httpCache: 42
  batch:
    delay: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  post(id: Int!): Post @http(url: "http://jsonplaceholder.typicode.com/posts/{{.args.id}}")
}

type User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.blog.userId}}") {
  id: Int!
  username: String!
  blog: Blog!
}

type Post
  @http(
    url: "http://jsonplaceholder.typicode.com/posts"
    query: [{key: "id", value: "{{.value.id}}"}]
    batchKey: ["blog", "blogId"]
  ) {
  id: Int!
  blog: Blog!
}

type Account
  @http(
    url: "http://jsonplaceholder.typicode.com/posts"
    query: [{key: "id", value: "{{.value.type.id}}"}]
    batchKey: ["blog", "blogId"]
  ) {
  id: Int!
  balance: Blog!
  type: AccountType
}

type Blog @http(url: "http://jsonplaceholder.typicode.com/posts", query: [{key: "id", value: "{{.value.blogId}}"}]) {
  id: Int!
}
```
