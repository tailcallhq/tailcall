# Batching group by

```graphql @server
schema @server(port: 8000, queryValidation: false) @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {delay: 1, maxSize: 1000}, httpCache: true) {
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
      - body: bar
        id: 11
        title: foo
        userId: 1
      - body: bar
        id: 3
        title: foo
        userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&foo=bar&id=2&foo=bar
    body: null
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
      - id: 2
        name: Ervin Howell
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { id } userId } }
```
