# test-upstream-headers

```graphql @server
schema @server @upstream(allowedHeaders: ["X-bar", "x-foo"], baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Post {
  id: Int
}

type Query {
  posts: [Post] @http(path: "/posts")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
    headers:
      x-foo: bar
      x-bar: baz
    body: null
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
```

```yml @assert
- method: POST
  url: http://localhost:8000/graphql
  headers:
    X-foo: bar
    X-bar: baz
  body:
    query: query { posts { id } }
```
