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
  bar: FooBar @discriminate
  foo: FooBar @discriminate
}

type Query {
  arr: [FooBar] @http(url: "http://jsonplaceholder.typicode.com/arr") @discriminate
  bar: FooBar @http(url: "http://jsonplaceholder.typicode.com/bar") @discriminate
  foo: FooBar @http(url: "http://jsonplaceholder.typicode.com/foo") @discriminate
  nested: Nested @http(url: "http://jsonplaceholder.typicode.com/nested")
  unknown: FooBar @http(url: "http://jsonplaceholder.typicode.com/unknown") @discriminate
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
      foo: test-foo
      type: "Foo"

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar
  expectedHits: 2
  response:
    status: 200
    body:
      bar: test-bar
      type: "Bar"

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/nested
  expectedHits: 2
  response:
    status: 200
    body:
      foo:
        foo: nested-foo
        type: "Foo"
      bar:
        bar: nested-bar
        type: "Bar"

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/arr
  expectedHits: 2
  response:
    status: 200
    body:
      - foo: foo1
        type: "Foo"
      - bar: bar2
        type: "Bar"
      - foo: foo3
        type: "Foo"
      - foo: foo4
        type: "Foo"
      - bar: bar5
        type: "Bar"

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/unknown
  response:
    status: 200
    body:
      unknown: baz
      type: "Unknown"
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
