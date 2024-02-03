# Js Request Response Hello World

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

#### mock:

```yml
- request:
    method: GET
    url: http://localhost:3000/bye
    body: '""'
  response:
    status: 200
    body: hello world
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { hi }
```
