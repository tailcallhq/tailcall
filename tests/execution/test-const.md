# test-const

###### check identity

####

```graphql @server
schema @server @upstream {
  query: Query
}

type Query {
  hello: String @const(data: "Hello from server")
}
```
