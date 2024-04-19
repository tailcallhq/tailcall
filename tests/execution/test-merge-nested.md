# test-merge-nested

```graphql @server
schema @server @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  hi: Foo @expr(body: "world")
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
  hi: Foo @expr(body: {a: "world"})
}

type Foo {
  """
  test2
  """
  a: String
}
```
