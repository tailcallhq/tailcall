# Batching default

```yaml @config
server:
  port: 8000
upstream:
  httpCache: 42
  batch:
    delay: 1
```

```graphql @schema
schema {
  query: Query
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
  users: [User] @http(url: "http://jsonplaceholder.typicode.com/users")
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  foo: [Foo] @http(url: "http://jsonplaceholder.typicode.com/foo")
}

type Foo {
  a: Int
  b: Int
  bar: Bar
    @http(url: "http://jsonplaceholder.typicode.com/bar", method: POST, body: {id: "{{.value.a}}"}, batchKey: ["a"])
}

type Bar {
  a: Int
  b: Int
}

type User {
  id: Int!
  name: String!
  email: String!
  post: Post
    @http(
      url: "http://jsonplaceholder.typicode.com/posts"
      method: POST
      body: {userId: "{{.value.id}}", title: "title", body: "body"}
      batchKey: ["userId"]
    )
}

type Post {
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      method: POST
      body: {key: "id", value: "{{.value.userId}}"}
      batchKey: ["id"]
    )
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    body:
      - id: 1
        name: user-1
        email: user-1@gmail.com
      - id: 2
        name: user-2
        email: user-2@gmail.com
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/posts
    body: [{"userId": "1", "title": "title", "body": "body"}, {"userId": "2", "title": "title", "body": "body"}]
  response:
    status: 200
    body:
      - id: 1
        userId: 1
        title: user-1
        body: user-1@gmail.com
      - id: 2
        userId: 2
        title: user-2
        body: user-2@gmail.com

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - id: 1
        userId: 1
        title: user-1
        body: user-1@gmail.com
      - id: 2
        userId: 2
        title: user-2
        body: user-2@gmail.com

- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/users
    body: [{"key": "id", "value": "1"}, {"key": "id", "value": "2"}]
  response:
    status: 200
    body:
      - id: 1
        name: user-1
        email: user-1@gmail.com
      - id: 2
        userId: 2
        name: user-2
        email: user-2@gmail.com

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foo
  expectedHits: 1
  response:
    status: 200
    body:
      - a: 11
        b: 12
      - a: 21
        b: 22

- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/bar
    body: [{"id": "11"}, {"id": "21"}]
  response:
    status: 200
    body:
      - a: 11
        b: 12
      - a: 21
        b: 22
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id name post { id title userId } } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { id title user { id name } } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { foo { a b bar { a  b } } }
```
