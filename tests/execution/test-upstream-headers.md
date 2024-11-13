# test-upstream-headers

```graphql @config
schema {
  query: Query
}
type Query {
  posts: [Post] @http(url: "http://jsonplaceholder.typicode.com/posts")
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

```yml @file:config.yml
schema: {}
upstream:
  allowedHeaders: ["x-foo", "X-bar"]
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  headers:
    X-foo: bar
    X-bar: baz
  body:
    query: query { posts { id } }
```
