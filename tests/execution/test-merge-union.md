# test-merge-union

```graphql @schema
schema @server {
  query: Query
}

union FooBar = Bar | Foo

type Bar {
  bar: String
}

type Foo {
  foo: String
}

type Query {
  foo: FooBar @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```

```graphql @schema
schema @server {
  query: Query
}

union FooBar = Baz | Foo

type Baz {
  baz: String
}

type Foo {
  foo: String
  a: String
}

type Query {
  foo: FooBar @http(url: "http://jsonplaceholder.typicode.com/foo")
}
```
