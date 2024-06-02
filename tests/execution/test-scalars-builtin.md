# Test builtin GraphQL scalars

```graphql @config
schema @server(port: 8000, hostname: "localhost") {
  query: Query
}

type Query {
  int(x: Int!): Int! @expr(body: "{{.args.x}}")
  float(x: Float!): Float! @expr(body: "{{.args.x}}")
  string(x: String!): String! @expr(body: "{{.args.x}}")
  bool(x: Boolean!): Boolean! @expr(body: "{{.args.x}}")
  id(x: ID!): ID @expr(body: "{{.args.x}}")
}
```

```yml @test
# Valid value tests
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: int(x: 2485165) b: int(x: -543521) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: float(x: 1.256) b: float(x: -15651) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: string(x: "str") b: string(x: "15616") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: bool(x: true) b: bool(x: false) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: id(x: "test-id") b: id(x: "123") }'

# Invalid value test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: int(x: "2485165") b: int(x: "str") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: float(x: true) b: float(x: "str") }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: string(x: true) b: string(x: 123) }"

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ a: bool(x: "true") b: bool(x: 0) }'

- method: POST
  url: http://localhost:8000/graphql
  body:
    query: "{ a: id(x: true) b: id(x: 1.25) }"
```
