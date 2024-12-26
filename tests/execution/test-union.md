---
identity: true
---

# Test union type resolve

```graphql @schema
schema @server @upstream {
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
  arr: [FooBar] @http(url: "http://jsonplaceholder.typicode.com/arr")
  bar: FooBar @http(url: "http://jsonplaceholder.typicode.com/bar")
  foo: FooBar @http(url: "http://jsonplaceholder.typicode.com/foo")
  nested: Nested @http(url: "http://jsonplaceholder.typicode.com/nested")
  unknown: FooBar @http(url: "http://jsonplaceholder.typicode.com/unknown")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foo
  expectedHits: 2
  response:
    status: 200
    body:
      Foo:
        foo: test-foo

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar
  expectedHits: 2
  response:
    status: 200
    body:
      Bar:
        bar: test-bar

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/nested
  expectedHits: 2
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
  expectedHits: 2
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
      Unknown:
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

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        foo {
          __typename
        }
        bar {
          __typename
        }
        arr {
          __typename
        }
        nested {
          foo {
            __typename
          }
          bar {
            __typename
          }
        }
      }
```
