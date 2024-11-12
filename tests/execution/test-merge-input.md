# Test merge input

```graphql @config
schema {
  query: Query
}

input Test {
  a: Int
  b: String
}

type Query {
  foo(x: Test): Boolean @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```graphql @config
schema {
  query: Query
}

input Test {
  b: String
  c: Boolean
}

type Query {
  foo(x: Test): Boolean @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
