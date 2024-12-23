# test-alias-on-enum

```graphql @schema
schema @server(batchRequests: true) @upstream(batch: {delay: 1, headers: [], maxSize: 100}) {
  query: Query
}

schema @server(enableJIT: false) {
  query: Query
}

enum Department {
  ENGINEERING
  MARKETING
  HUMAN_RESOURCE @alias(options: ["HR"])
}

type Query {
  color: DTA @expr(body: {departments: ["ENGINEERING", "MARKETING", "HR"]})
}

type DTA {
  departments: [Department]
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { color { departments } }"
```
