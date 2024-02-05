# expr logic

#### server:

```graphql
schema {
  query: Query
}

type Query {
  median_err_zero: Int @expr(body: {median: [{const: -1}, {const: 0}, {const: 1}]})
  median_err_negative: Int @expr(body: {median: [{const: -1}, {const: -2}, {const: -3}]})
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { median_err_zero }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { median_err_negative }
```
