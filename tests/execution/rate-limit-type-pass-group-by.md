# Rate Limit Field

#### server:

```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  bars: [Bar] @http(path: "/bars")
}

type Bar @rateLimit(requestsPerUnit: 3, unit: "hour", groupBy: "id") {
  id: Int
  name: String
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://example.com/bars
    body: null
  response:
    status: 200
    body:
      - id: 1
      - id: 1
      - id: 2
      - id: 1
      - id: 2
      - id: 2
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { bars { id } }
```
