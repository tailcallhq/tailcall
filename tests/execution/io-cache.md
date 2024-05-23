# Call operator with GraphQL data source

```graphql @config
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type User {
  id: Int!
  name: String!
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(path: "/users/{{.value.userId}}")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - id: 1
        title: a
        userId: 1
      - id: 2
        title: b
        userId: 1
      - id: 3
        title: c
        userId: 2
      - id: 4
        title: d
        userId: 2
      - id: 5
        title: e
        userId: 2
      - id: 6
        title: f
        userId: 2
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 2
  response:
    status: 200
    body:
      name: Leanne Graham
- request:
    url: http://jsonplaceholder.typicode.com/users/2
  expectedHits: 4
  response:
    status: 200
    body:
      name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
