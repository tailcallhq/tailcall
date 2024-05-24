---
identity: true
---

# test-union

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

scalar Baz

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

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foo
  response:
    status: 200
    body:
      foo: test-foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        foo {
          ... on Foo {
            foo
          }
          ... on Bar {
            bar
          }
        }
      }
```
