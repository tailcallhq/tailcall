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
  hi: Foo @expr(body: "world")
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
  hi: Foo @expr(body: {a: "world"})
}
```
