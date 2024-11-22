# Batching default

```graphql @config
schema @server(port: 8000) @upstream(httpCache: 42) {
  query: Query
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}

type User {
  id: Int!
  name: String!
  email: String!
  post: Post @http(url: "http://jsonplaceholder.typicode.com/posts", method: POST, body: [{userId: "{{.value.id}}", title: "{{.value.name}}", body: "{{.value.email}}"}], batchKey: ["id"])
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    body:
      - id: 1
        name: user-1
        email: user-1@gmail.com
      - id: 2
        name: user-2
        email: user-2@gmail.com
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: [{"userId": "1", "title": "user-1", "body": "user-1@gmail.com"},{"userId": "2", "title": "user-2", "body": "user-2@gmail.com"}]
  response:
    status: 200
    body:
      - id: 1
        userId: 1
        title: user-1
        body: user-1@gmail.com
      - id: 2
        userId: 2
        title: user-2
        body: user-2@gmail.com
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id name post { id title userId } } }
```
