# test-file-not-found

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
  user: User @file(src: "./doesntexist.json")
}
```
