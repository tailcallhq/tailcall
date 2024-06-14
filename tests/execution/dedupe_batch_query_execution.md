# Async Cache Inflight Enabled

```graphql @config
schema
  @server(port: 8000, queryValidation: false, dedupe: true)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts?id=1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts?id=1
  response:
    status: 200
    body:
      - id: 1
        userId: 1
      - id: 2
        userId: 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { id, userId } }
```
