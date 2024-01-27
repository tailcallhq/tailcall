# Batching post

#### server:

```graphql
schema
  @server(port: 8000, queryValidation: false)
  @upstream(
    httpCache: true
    batch: {maxSize: 1000, delay: 1, headers: []}
    baseURL: "http://jsonplaceholder.typicode.com"
  ) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts?id=1")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User @http(path: "/users/{{value.userId}}")
}

type User {
  id: Int
  name: String
}
```

#### assert:

```yml
mock:
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/posts?id=1
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        - id: 1
          userId: 1
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        id: 1
        name: Leanne Graham
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { posts { user { name } } }
env: {}
```
