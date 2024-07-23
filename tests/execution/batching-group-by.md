# Batching group by

```graphql @config
schema
  @server(port: 8000, queryValidation: false)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts?id=11&id=3&foo=1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User
    @http(
      path: "/users"
      query: [{key: "id", value: "{{.value.userId}}"}, {key: "foo", value: "bar"}]
      batchKey: ["id"]
    )
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
    url: http://jsonplaceholder.typicode.com/users?id=1&foo=bar&id=2
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
      - id: 2
        name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { id } userId } }
```
