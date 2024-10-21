# test-merge-union

```graphql @config
schema @server  {
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

```graphql @config
schema @server  {
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
