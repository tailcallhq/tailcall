# test-enum-merge

```graphql @schema
schema @server {
  query: Query
}

enum Foo {
  BAR
  BAZ
}

type Query {
  foo: Foo @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```graphql @schema
schema @server {
  query: Query
}

enum Foo {
  BAR
  BOOM
}

type Query {
  foo: Foo @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
