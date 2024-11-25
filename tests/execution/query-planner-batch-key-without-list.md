# Query Planner Batching Without the List Ancestor or Batch Setting Enabled.

```graphql @config
schema @server(port: 8000) @upstream(httpCache: 42) {
  query: Query
}

type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int
  name: String
  post: Post
    @http(
      url: "https://jsonplaceholder.typicode.com/posts"
      query: [{key: "id", value: "{{.value.id}}"}]
      batchKey: ["id"]
    )
}

type Post {
  id: Int
  userId: Int
  title: String
  user: User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.value.userId}}"}]
      batchKey: ["id"]
    )
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: user-1

- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/posts?id=1
  response:
    status: 200
    body:
      - id: 1
        userId: 1
        title: post-1

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - id: 1
        userId: 1
        title: post-1
      - id: 2
        userId: 2
        title: post-2

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
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
  body:
    query: query { user { id name post { id title } } }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { posts { id userId user { id name } } }
```
