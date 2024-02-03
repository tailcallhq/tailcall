# expr concat

#### server:

```graphql
schema {
  query: Query
}

type Query {
  concat: [Int] @expr(body: {concat: [{const: [1, 2]}, {const: [3, 4]}]})
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { concat }
```
