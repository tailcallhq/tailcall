# expr intersection

```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  intersection: [Int] @expr(body: {intersection: [{const: [1, 2, 3]}, {const: [3, 4, 5]}]})
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { intersection }
```
