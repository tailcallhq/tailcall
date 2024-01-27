# Js Hello World

#### server:
```graphql
schema @server(script: {path: {src: "tests/http/scripts/test.js"}}) {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```

#### assert:
```yml
mock: []
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { hello }
env: {}
```
