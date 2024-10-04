# Test merge input

```graphql @config
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

input Test {
  a: Int
  b: String
}

type Query {
  foo(x: Test): Boolean @http(path: "/foo")
}
```

```graphql @config
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

input Test {
  b: String
  c: Boolean
}

type Query {
  foo(x: Test): Boolean @http(path: "/foo")
}
```
