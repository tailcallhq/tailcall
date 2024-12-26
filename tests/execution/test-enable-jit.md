# test-enable-jit

```yaml @config
server:
  port: 8000
  hostname: "0.0.0.0"
  enableJIT: true
```

```graphql @schema
schema {
  query: Query
}

type Query @cache(maxAge: 30000) {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
}

type User {
  id: Int!
  name: String!
  username: String!
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/{{.value.userId}}")
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
      - body: bar
        id: 3
        title: foo
        userId: 2

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: foo
      username: foo
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
  response:
    status: 200
    body:
      id: 2
      name: bar
      username: bar
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: query { posts { id user { name } } }
```
