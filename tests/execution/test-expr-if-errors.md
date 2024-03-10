# test-expr-if-errors

###### sdl error

####
```graphql @server
schema @server {
  query: Query
}

type Query {
  noCond: String @expr(body: {if: {then: {const: "True"}, else: {const: "False"}}})
  noThen: String @expr(body: {if: {cond: {const: true}, else: {const: "False"}}})
  noElse: String @expr(body: {if: {cond: {const: true}, then: {const: "True"}}})
}
```
