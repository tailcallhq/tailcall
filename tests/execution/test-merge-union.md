# test-merge-union.graphql

#### server:

```graphql
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

#### server:

```graphql
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

#### merged:

```graphql
schema @server @upstream(baseURL: "http://jsonplacheholder.typicode.com") {
  query: Query
}

union FooBar = Bar | Baz | Foo

type Bar {
  bar: String
}

type Baz {
  baz: String
}

type Foo {
  a: String
  foo: String
}

type Query {
  foo: FooBar @http(path: "/foo")
}
```
