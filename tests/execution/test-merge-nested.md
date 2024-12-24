# test-merge-nested

```graphql @schema
schema @server {
  query: Query
}

type Query {
  hi: Foo @expr(body: {b: "hello"})
}

type Foo {
  """
  test1
  """
  b: String
}
```

```graphql @schema
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
