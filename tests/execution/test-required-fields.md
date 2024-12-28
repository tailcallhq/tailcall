# Test API

```graphql @schema
schema @server(enableJIT: true) {
  query: Query
}

type Query {
  basicPresent: Foo! @http(url: "http://jsonplaceholder.typicode.com/basic-present")
  basicFieldMissing: Foo! @http(url: "http://jsonplaceholder.typicode.com/basic-field-missing")
  basicMissing: Foo! @http(url: "http://jsonplaceholder.typicode.com/basic-missing")
  relaxedPresent: Foo @http(url: "http://jsonplaceholder.typicode.com/relaxed-present")
  relaxedFieldMissing: Foo @http(url: "http://jsonplaceholder.typicode.com/relaxed-field-missing")
  relaxedMissing: Foo @http(url: "http://jsonplaceholder.typicode.com/relaxed-missing")
  fullPresent: [Foo!]! @http(url: "http://jsonplaceholder.typicode.com/full-present")
  fullMissing: [Foo!]! @http(url: "http://jsonplaceholder.typicode.com/full-missing")
  fullFieldMissing: [Foo!]! @http(url: "http://jsonplaceholder.typicode.com/full-field-missing")
  fullEntryMissing: [Foo!]! @http(url: "http://jsonplaceholder.typicode.com/full-entry-missing")
  innerPresent: [Foo!] @http(url: "http://jsonplaceholder.typicode.com/inner-present")
  innerMissing: [Foo!] @http(url: "http://jsonplaceholder.typicode.com/inner-missing")
  innerFieldMissing: [Foo!] @http(url: "http://jsonplaceholder.typicode.com/inner-field-missing")
  innerEntryMissing: [Foo!] @http(url: "http://jsonplaceholder.typicode.com/inner-entry-missing")
  outerPresent: [Foo]! @http(url: "http://jsonplaceholder.typicode.com/outer-present")
  outerMissing: [Foo]! @http(url: "http://jsonplaceholder.typicode.com/outer-missing")
  outerFieldMissing: [Foo]! @http(url: "http://jsonplaceholder.typicode.com/outer-field-missing")
  outerEntryMissing: [Foo]! @http(url: "http://jsonplaceholder.typicode.com/outer-entry-missing")
  nonePresent: [Foo] @http(url: "http://jsonplaceholder.typicode.com/none-present")
  noneMissing: [Foo] @http(url: "http://jsonplaceholder.typicode.com/none-missing")
  noneFieldMissing: [Foo] @http(url: "http://jsonplaceholder.typicode.com/none-field-missing")
  noneEntryMissing: [Foo] @http(url: "http://jsonplaceholder.typicode.com/none-entry-missing")
}

type Foo {
  id: Int!
  bar: String!
}
```

```yml @mock
# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/basic-present
  response:
    status: 200
    body:
      id: 1
      bar: bar_1

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/basic-field-missing
  response:
    status: 200
    body:
      id: 1
      bar: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/basic-missing
  response:
    status: 200
    body: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/relaxed-present
  response:
    status: 200
    body:
      id: 1
      bar: bar_1

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/relaxed-field-missing
  response:
    status: 200
    body:
      id: 1
      bar: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/relaxed-missing
  response:
    status: 200
    body: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/full-present
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: bar_2

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/full-missing
  response:
    status: 200
    body: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/full-field-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/full-entry-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/inner-present
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: bar_2

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/inner-missing
  response:
    status: 200
    body: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/inner-field-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/inner-entry-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/outer-present
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: bar_2

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/outer-missing
  response:
    status: 200
    body: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/outer-field-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/outer-entry-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-present
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: bar_2

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-missing
  response:
    status: 200
    body: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-field-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - id: 2
        bar: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-entry-missing
  response:
    status: 200
    body:
      - id: 1
        bar: bar_1
      - null
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { basicPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { basicFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { basicMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { relaxedPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { relaxedFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { relaxedMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { fullPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { fullMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { fullFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { fullEntryMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { innerPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { innerMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { innerFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { innerEntryMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { outerPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { outerMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { outerFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { outerEntryMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { nonePresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { noneMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { noneFieldMissing { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { noneEntryMissing { id bar } }
```
