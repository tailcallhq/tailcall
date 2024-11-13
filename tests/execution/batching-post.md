# Batching post

```graphql @config
schema @link(src: "config.yml", type: Config) {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts?id=1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
}

type User {
  id: Int
  name: String
}
```

```yml @file:config.yml
schema: {}
server:
  port: 8000
  queryValidation: false
upstream:
  httpCache: 42
  batch:
    delay: 1
    maxSize: 1000
    headers: []
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
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { name } } }
```
