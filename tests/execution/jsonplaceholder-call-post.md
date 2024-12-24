# jsonplaceholder-call-post

```yaml @config
server:
  port: 8000
  hostname: "0.0.0.0"
upstream:
  httpCache: 42
  batch:
    delay: 100
```

```graphql @schema
schema {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
  user(id: Int!): User @http(url: "http://jsonplaceholder.typicode.com/users/{{.args.id}}")
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
