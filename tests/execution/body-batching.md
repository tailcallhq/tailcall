# Batching post

```yaml @config
server:
  port: 8000
  queryValidation: false
upstream:
  httpCache: 42
  batch:
    delay: 1
    maxSize: 1000
```

```graphql @schema
schema {
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
      body: {userId: "{{.value.id}}", title: "foo", body: "bar"}
      batchKey: ["userId"]
    )
  comments: [Comment]
    @http(
      url: "https://jsonplaceholder.typicode.com/comments"
      method: POST
      body: {title: "foo", body: "bar", meta: {information: {userId: "{{.value.id}}"}}}
      batchKey: ["userId"]
    )
}

type Comment {
  id: Int
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  expectedHits: 2
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
        {"userId": "1", "title": "foo", "body": "bar"},
        {"userId": "2", "title": "foo", "body": "bar"},
        {"userId": "3", "title": "foo", "body": "bar"},
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

- request:
    method: POST
    url: https://jsonplaceholder.typicode.com/comments
    body:
      [
        {"title": "foo", "body": "bar", "meta": {"information": {"userId": "1"}}},
        {"title": "foo", "body": "bar", "meta": {"information": {"userId": "2"}}},
        {"title": "foo", "body": "bar", "meta": {"information": {"userId": "3"}}},
      ]
  response:
    status: 200
    body:
      - id: 1
        userId: 1
      - id: 2
        userId: 2
      - id: 3
        userId: 3
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id posts { userId title } } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id comments { id } } }
```
