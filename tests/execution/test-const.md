# test-const

###### check identity

#### server:

```graphql
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "Hello from server")
}
```
