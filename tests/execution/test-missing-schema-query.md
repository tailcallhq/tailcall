# test-missing-schema-query

###### sdl error

```graphql @server
schema {
  mutation: Mutation
}

type Mutation {
  id: Int!
}
```
