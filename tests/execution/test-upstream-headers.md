# test-upstream-headers

```yaml @config
upstream:
  allowedHeaders: ["x-foo", "X-bar"]
```

```graphql @schema
schema {
  query: Query
}
type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
  fooField: String @expr(body: "{{.headers.x-foo}}")
  barField: String @expr(body: "{{.headers.x-bar}}")
}
type Post {
  id: Int
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
    headers:
      x-foo: bar
      x-bar: baz
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

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  headers:
    X-foo: bar
    X-bar: baz
  body:
    query: query { posts { id } fooField barField }
```
