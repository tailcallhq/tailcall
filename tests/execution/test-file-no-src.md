# test-file-no-src

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type User {
  name: String
}

type Query {
  user: User @file
}
```
