---
identity: true
---

# Test union type resolve

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

union FooBar = Bar | Foo

type Bar {
  bar: String!
}

type Foo {
  foo: String!
}

type Nested {
  bar: FooBar
  foo: FooBar
}

type Query {
  arr: [FooBar] @http(path: "/arr")
  bar: FooBar @http(path: "/bar")
  foo: FooBar @http(path: "/foo")
  nested: Nested @http(path: "/nested")
  unknown: FooBar @http(path: "/unknown")
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
```
