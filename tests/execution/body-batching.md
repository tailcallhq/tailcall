# Batching post

```graphql @config
schema
  @server(port: 8000, queryValidation: false)
  @upstream(httpCache: 42, batch: {maxSize: 1000, delay: 1, headers: []}) {
  query: Query
}

type Query {
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
}

type User {
  id: Int!
  name: String!
  posts: [Post]
    @http(
      url: "https://jsonplaceholder.typicode.com/posts"
      method: POST
      body: "{\"userId\":\"{{.value.id}}\",\"title\":\"foo\",\"body\":\"bar\"}"
      batchKey: ["userId"]
      bodyKey: ["userId"]
    )
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
      - id: 2
        name: user-2
      - id: 3
        name: user-3
- request:
    method: POST
    url: https://jsonplaceholder.typicode.com/posts
    body:
      [
        {"userId":"1","title":"foo","body":"bar"},
        {"userId":"2","title":"foo","body":"bar"},
        {"userId":"3","title":"foo","body":"bar"},
      ]
  response:
    status: 200
    body:
      - id: 1
        title: foo
        body: bar
        userId: 1
      - id: 2
        title: foo
        body: bar
        userId: 2
      - id: 3
        title: foo
        body: bar
        userId: 3
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id posts { userId title } } }
```
