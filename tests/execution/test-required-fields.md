# Test API

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  basicPresent: Foo! @http(path: "/basic-present")
  basicMissing: Foo! @http(path: "/basic-missing")
  relaxedPresent: Foo @http(path: "/relaxed-present")
  relaxedMissing: Foo @http(path: "/relaxed-missing")
  fullPresent: [Foo!]! @http(path: "/full-present")
  fullMissing: [Foo!]! @http(path: "/full-missing")
  innerPresent: [Foo!] @http(path: "/inner-present")
  innerMissing: [Foo!] @http(path: "/inner-missing")
  outerPresent: [Foo]! @http(path: "/outer-present")
  outerMissing: [Foo]! @http(path: "/outer-missing")
  nonePresent: [Foo] @http(path: "/none-present")
  noneMissing: [Foo] @http(path: "/none-missing")
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
    url: http://jsonplaceholder.typicode.com/basic-missing
  response:
    status: 200
    body:
      id: 1
      bar: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/relaxed-present
  response:
    status: 200
    body:
      id: 1
      bar: bar_1

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
    body: null

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/inner-missing
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
    url: http://jsonplaceholder.typicode.com/outer-present
  response:
    status: 200
    body: [null]

# this fails
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/outer-missing
  response:
    status: 200
    body: null

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-present
  response:
    status: 200
    body: []

# this does not fail
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/none-missing
  response:
    status: 200
    body: null
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { basicPresent { id bar } }
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
    query: query { innerPresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { innerMissing { id bar } }
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
    query: query { nonePresent { id bar } }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { noneMissing { id bar } }
```
