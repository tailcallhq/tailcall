# Batching group by

```graphql @config
schema
  @server(port: 8000, queryValidation: false)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int
  user: User
    @http(
      path: "/users"
      query: [{key: "id", value: "{{.value.userId}}", skipEmpty: true}, {key: "foo", value: "bar"}]
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
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - body: bar
        id: 11
        title: foo
        userId: 1
      - body: bar # no userId for bar
        id: 3
        title: foo
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&foo=bar
  response:
    status: 200
    body:
        id: 1
        name: Leanne Graham

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?foo=bar
  response:
    status: 200
    body:
        id: 2
        name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { user { id } userId } }
```
