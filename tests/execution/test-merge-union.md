# test-merge-union

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
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
  foo: FooBar @http(path: "/foo")
}
```

```graphql @server
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
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
  foo: FooBar @http(path: "/foo")
}
```
