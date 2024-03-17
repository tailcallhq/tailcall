# test-merge-nested

```graphql @server
schema @server @upstream(baseURL: "http://abc.com") {
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
schema @server @upstream {
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
