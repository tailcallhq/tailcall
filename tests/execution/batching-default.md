# Batching default

```graphql @server
schema @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {delay: 10, maxSize: 100}, httpCache: true) {
  query: Query
}

type Post {
  body: String
  id: Int
  title: String
  user: User @http(batchKey: ["id"], path: "/users", query: [{key: "id", value: "{{value.userId}}"}, {key: "foo", value: "bar"}])
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts?id=11&id=3&foo=1")
}

type User {
  id: Int
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts?id=11&id=3&foo=1
    body: null
  response:
    status: 200
    body:
      - id: 11
        userId: 1
      - id: 3
        userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&foo=bar&id=2&foo=bar
    body: null
  response:
    status: 200
    body:
      - id: 1
      - id: 2
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { id } userId } }
```
