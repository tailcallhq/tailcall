# test-merge-nested

```graphql @schema
schema {
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

```graphql @schema
schema {
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
