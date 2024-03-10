# Batching inside nested @expr

####
```graphql @server
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true, batch: {delay: 10}) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int!
  user: User
    @expr(
      body: {
        if: {
          cond: {const: {data: true}}
          then: {http: {path: "/users", query: [{key: "id", value: "{{value.userId}}"}], groupBy: ["id"]}}
          else: {const: {data: {}}}
        }
      }
    )
}

type User {
  id: Int
  name: String
  values: [Value]
    @expr(
      body: {
        concat: [
          {http: {path: "/users-values-1", query: [{key: "id", value: "{{value.id}}"}], groupBy: ["id"]}}
          {http: {path: "/users-values-2", query: [{key: "id", value: "{{value.id}}"}], groupBy: ["id"]}}
        ]
      }
    )
}

type Value {
  value: Int
}
```

####
```yml @mock
- request:
    url: http://jsonplaceholder.typicode.com/posts
  response:
    body:
      - id: 11
        userId: 1
      - id: 3
        userId: 2
- request:
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2
  response:
    body:
      - id: 1
      - id: 2
- request:
    url: http://jsonplaceholder.typicode.com/users-values-1?id=1&id=2
  response:
    body:
      - {id: 1, value: 1}
      - {id: 2, value: 6}
      - {id: 2, value: 7}
- request:
    url: http://jsonplaceholder.typicode.com/users-values-2?id=1&id=2
  response:
    body:
      - {id: 1, value: 2}
      - {id: 1, value: 3}
      - {id: 2, value: 8}
      - {id: 2, value: 9}
```

####
```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { user { id, values { value } } } }"
```
