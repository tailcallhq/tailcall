# GraphQL Request from JSON file source

#### server:

```graphql
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  users: [User] @file(src: "./tests/http/config/users.json")
  expr_users: [User] @expr(body: {file: {src: "./tests/http/config/users.json"}})
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { users { id name } expr_users { id name } }
```
