# test-merge-nested

```graphql @server
schema @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Foo {
  """
  test1
  """
  b: String
}

type Query {
  hi: Foo @const(data: "world")
}
```

```graphql @server
schema {
  query: Query
}

type Foo {
  """
  test2
  """
  a: String
}

type Query {
  hi: Foo @const(data: {a: "world"})
}
```
