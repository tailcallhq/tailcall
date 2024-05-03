# test-expr

```graphql @server
schema {
  query: Query
}

type Query {
  hello: String @expr(body: "Hello from server")
}
```
