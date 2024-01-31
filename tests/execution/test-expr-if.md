# test-expr-if

#### server:

```graphql
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  ifFalse: String @expr(body: {if: {cond: {const: false}, then: {const: "pass"}, else: {const: "fail"}}})
  ifTrue: String @expr(body: {if: {cond: {const: true}, then: {const: "pass"}, else: {const: "fail"}}})
}
```

#### query:

```graphql
query {
  ifFalse
  ifTrue
}
```
