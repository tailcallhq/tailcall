# expr concat

####

```graphql @server
schema {
  query: Query
}

type Query {
  concat: [Int] @expr(body: {concat: [{const: [1, 2]}, {const: [3, 4]}]})
}
```

####

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { concat }
```
