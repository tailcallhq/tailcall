# jsonplaceholder-call-post

```graphql @config
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42, batch: {delay: 100}) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  users: [User] @http(path: "/users")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
}

type User {
  id: Int!
  name: String!
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])
}
```

```yml @mock
- request:
    url: http://jsonplaceholder.typicode.com/posts
  expectedHits: 1
  response:
    body:
      - id: 1
        title: title1
        userId: 1
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 1
  response:
    body:
      id: 1
      name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
