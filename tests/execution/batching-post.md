# Batching post

```graphql @server
schema @server(port: 8000, queryValidation: false) @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {delay: 1, headers: [], maxSize: 1000}, httpCache: true) {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @http(path: "/users/{{value.userId}}")
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts?id=1")
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
    body: null
  response:
    status: 200
    body:
      - id: 1
        userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { name } } }
```
