# Test expr with mustache

```graphql @schema
schema {
  query: Query
}

enum Department {
  ENGINEERING
  MARKETING
  BLUE
}

type Query {
  color: DTA @expr(body: {departments: ["ENGINEERING", "MARKETING"]})
}

type DTA {
  departments: [Department]
}
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: |
      {
        color {
          departments
        }
      }
```
