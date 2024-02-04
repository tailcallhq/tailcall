# test-expr-errors

###### sdl error

#### server:

```graphql
schema @server {
  query: Query
}

type Query {
  foo: String @expr(data: {const: "John"})
  bar: String @expr(body: {unsupported: true})
}
```
