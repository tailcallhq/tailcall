# Call operator with GraphQL data source

```graphql @server
schema @server(graphiql: true, hostname: "0.0.0.0", port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) {
  query: Query
}

type Post {
  body: String!
  id: Int!
  title: String!
  user: User @http(path: "/users/{{value.userId}}")
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type User {
  id: Int!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
    body: null
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
  expected_hits: 1
  response:
    status: 200
    body:
      name: Leanne Graham
- request:
    url: http://jsonplaceholder.typicode.com/users/2
  expected_hits: 1
  response:
    status: 200
    body:
      name: Ervin Howell
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
