# Test union types in ambiguous case

In some cases, when the resolved data shape does not strongly correspond to GraphQL types, the discriminator may return the first possible type or no possible types at all.

```graphql @schema
schema @server {
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
  arr: [FooBarBaz] @http(url: "http://jsonplaceholder.typicode.com/arr")
  bar: FooBarBaz @http(url: "http://jsonplaceholder.typicode.com/bar")
  foo: FooBarBaz @http(url: "http://jsonplaceholder.typicode.com/foo")
  nested: Nested @http(url: "http://jsonplaceholder.typicode.com/nested")
  unknown: FooBarBaz @http(url: "http://jsonplaceholder.typicode.com/unknown")
  wrong: FooBarBaz @http(url: "http://jsonplaceholder.typicode.com/wrong")
  string: FooBarBaz @http(url: "http://jsonplaceholder.typicode.com/string")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foo
  response:
    status: 200
    body:
      Foo:
        foo: test-foo

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar
  response:
    status: 200
    body:
      Bar:
        bar: test-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/nested
  response:
    status: 200
    body:
      foo:
        Foo:
          foo: nested-foo
      bar:
        Bar:
          bar: nested-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/arr
  response:
    status: 200
    body:
      - Foo:
          foo: foo1
      - Bar:
          bar: bar2
      - Foo:
          foo: foo3
      - Foo:
          foo: foo4
      - Bar:
          bar: bar5

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/unknown
  response:
    status: 200
    body:
      Bar:
        unknown: baz

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/wrong
  response:
    status: 200
    body:
      Baz:
        foo: test-foo
        bar: test-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/string
  response:
    status: 200
    body:
      Foo: "test-string"
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
          __typename
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        string {
          foo
          __typename
        }
      }
```
