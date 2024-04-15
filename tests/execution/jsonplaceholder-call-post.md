# jsonplaceholder-call-post

```graphql @server
schema @server(graphiql: true, hostname: "0.0.0.0", port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {delay: 100, headers: [], maxSize: 100}, httpCache: true) {
  query: Query
}

type Post {
  id: Int!
  title: String!
  user: User @call(steps: [{query: "user", args: {id: "{{value.userId}}"}}])
  userId: Int!
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{args.id}}")
  users: [User] @http(path: "/users")
}

type User {
  id: Int!
  name: String!
}
```

```yml @mock
- request:
    url: http://jsonplaceholder.typicode.com/posts
  expected_hits: 1
  response:
    body:
      - id: 1
        title: title1
        userId: 1
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  expected_hits: 1
  response:
    body:
      id: 1
      name: Leanne Graham
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { title user { name } } }
```
