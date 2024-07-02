# Test union types in ambiguous case

In some cases, when the resolved data shape does not strongly correspond to GraphQL types, the discriminator may return the first possible type or no possible types at all.

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

union FooBarBaz = Bar | Foo | Baz

type Bar {
  bar: String
}

type Baz {
  bar: String
  baz: String
}

type Foo {
  foo: String
}

type Nested {
  bar: FooBarBaz
  foo: FooBarBaz
}

type Query {
  arr: [FooBarBaz] @http(path: "/arr")
  bar: FooBarBaz @http(path: "/bar")
  foo: FooBarBaz @http(path: "/foo")
  nested: Nested @http(path: "/nested")
  unknown: FooBarBaz @http(path: "/unknown")
  wrong: FooBarBaz @http(path: "/wrong")
  string: FooBarBaz @http(path: "/string")
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

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar
  response:
    status: 200
    body:
      bar: test-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/nested
  response:
    status: 200
    body:
      foo:
        foo: nested-foo
      bar:
        bar: nested-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/arr
  response:
    status: 200
    body:
      - foo: foo1
      - bar: bar2
      - foo: foo3
      - foo: foo4
      - bar: bar5

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/unknown
  response:
    status: 200
    body:
      unknown: baz

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/wrong
  response:
    status: 200
    body:
      foo: test-foo
      bar: test-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/string
  response:
    status: 200
    body: "test-string"
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
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        bar {
          ... on Bar {
            bar
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        nested {
          foo {
            ... on Foo {
              foo
            }
          }
          bar {
            ... on Bar {
              bar
            }
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        arr {
          ... on Foo {
            foo
          }
          ... on Bar {
            bar
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        unknown {
          ... on Foo {
            foo
          }
          ... on Bar {
            bar
          }
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        wrong {
          foo
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        string {
          foo
        }
      }
```
