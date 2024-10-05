---
error: true
---

# Apollo federation validation

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
  query: Query
}

type Query {
  post(id: Int!): Post @http(path: "/posts/{{.args.id}}")
}

type User @http(path: "/users/{{.value.blog.userId}}") {
  id: Int!
  username: String!
  blog: Blog!
}

type Post @http(path: "/posts", query: [{key: "id", value: "{{.value.id}}"}], batchKey: ["blog", "blogId"]) {
  id: Int!
  blog: Blog!
}

type Blog @http(path: "/posts", query: [{key: "id", value: "{{.value.blogId}}"}]) {
  id: Int!
}
```
