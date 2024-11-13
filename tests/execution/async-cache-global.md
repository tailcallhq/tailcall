# Async Cache Inflight Enabled

```graphql @config
schema {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts?id=1", dedupe: true)
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

```yml @file:config.yml
schema: {}
server:
  port: 8000
  queryValidation: false
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { id, userId } }
```
