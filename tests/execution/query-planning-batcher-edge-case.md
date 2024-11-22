# Batching group by

```graphql @config
schema @server(port: 8000, queryValidation: false) {
  query: Query
}

type Query {
  postData: [PostData] @http(url: "http://jsonplaceholder.typicode.com/posts?id=1&id=2")
}

type PostData {
  id: Int!
  meta: String @expr(body: "Data owned by tailcall.")
  post: Post @http(url: "http://jsonplaceholder.typicode.com/nested-posts", query: [{key: "id", value: "{{.value.id}}"}], batchKey: ["id"])
}

type Post {
  id: Int
  title: String
  body: String
  userId: Int
  user: User
    @http(
      url: "http://jsonplaceholder.typicode.com/users"
      query: [{key: "id", value: "{{.value.userId}}"}, {key: "foo", value: "bar"}]
      batchKey: ["id"]
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
    url: http://jsonplaceholder.typicode.com/posts?id=1&id=2
  response:
    status: 200
    body:
      - id: 1
      - id: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/nested-posts?id=1&id=2
  response:
    status: 200
    body:
      - id: 1
        title: post-1
        body: post-1
        userId: 1
      - id: 2
        title: post-2
        body: post-2
        userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1&id=2&foo=bar
  response:
    status: 200
    body:
      - id: 1
        name: Leanne Graham
      - id: 2
        name: Ervin Howell
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { postData { meta post { id user { id } } userId } }
```
