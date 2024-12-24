# Test merge input

```graphql @schema
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

```graphql @schema
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
