# test-merge-nested

```graphql @server
schema @server @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  hi: Foo @const(data: "world")
}

type Foo {
  """
  test1
  """
  b: String
}
```

```graphql @server
schema @server {
  query: Query
}

type Query {
  hi: Foo @const(data: {a: "world"})
}

type Foo {
  """
  test2
  """
  a: String
}
```
