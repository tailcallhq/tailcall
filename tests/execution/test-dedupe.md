# testing dedupe functionality

```yaml @config
server:
  port: 8000
upstream:
  batch:
    delay: 1
```

```graphql @schema
schema {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts?id=1", dedupe: true)
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.value.userId}}"}]
      batchKey: ["id"]
      dedupe: true
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
    url: http://jsonplaceholder.typicode.com/posts?id=1
  expectedHits: 1
  delay: 10
  response:
    status: 200
    body:
      - id: 1
        userId: 1
      - id: 2
        userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
  expectedHits: 1
  delay: 10
  response:
    status: 200
    body:
      - id: 1
        name: user-1
      - id: 2
        name: user-2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  concurrency: 10
  body:
    query: query { posts { id, userId user { id name } duplicateUser:user { id name } } }
```
