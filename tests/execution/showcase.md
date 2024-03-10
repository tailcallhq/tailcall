# Showcase GraphQL Request

####

```graphql @server
schema @server(showcase: true) {
  query: Query
}

type User {
  not_id: Int
  not_name: String
}

type Query {
  not_user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

####

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    headers:
      test: test
    body: null
  response:
    status: 200
    body:
      id: 1
      name: foo
- request:
    method: GET
    url: http://example.com/simple.graphql
    body: null
  expected_hits: 2
  response:
    status: 200
    textBody: |2-
       schema { query: Query }
      type User { id: Int name: String }
      type Query { user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com") }
- request:
    method: GET
    url: http://example.com/invalid.graphql
    body: null
  response:
    status: 200
    body: dsjfsjdfjdsfjkdskjfjkds
```

####

```yml @assert
- method: POST
  url: http://localhost:8080/showcase/graphql?config=http%3A%2F%2Fexample.com%2Fsimple.graphql
  body:
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/showcase/graphql
  body:
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/showcase/graphql?config=.%2Ftests%2Fhttp%2Fconfig%2Fsimple.graphql
  body:
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/showcase/graphql?config=http%3A%2F%2Fexample.com%2Finvalid.graphql
  body:
    query: query { user { name } }
- method: POST
  url: http://localhost:8080/showcase/graphql?config=http%3A%2F%2Fexample.com%2Fsimple.graphql
  body:
    query:
      foo: bar
```
