# Using @protected operator without specifying server.auth config

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type Query {
  data: String @const(data: "data") @protected
}
```
