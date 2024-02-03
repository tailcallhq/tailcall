# test-file

###### check identity

#### server:

```graphql
schema @server @upstream {
  query: Query
}

type Query {
  users: [User] @file(src: "./tests/http/config/users.json")
}

type User {
  id: Int
  name: String
}
```
