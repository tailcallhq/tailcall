# js_request-to-js-runtime

```graphql @server
schema @server(headers: {experimental: ["x-tailcall"]}) @upstream(baseURL: "http://example.com") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts?id=11")
}

type Post {
  id: Int
  title: String
  body: String
}
```

```yml @mock
- request:
    method: GET
    url: http://example.com/posts?id=11
    headers:
      X-tailcall: "tailcall-header"
    body: null
  response:
    status: 200
    body:
      - body: bar
        id: 11
        title: foo
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  headers:
    X-tailcall: "tailcall-header"
  body:
    query: query { posts { id, title, body } }
```
