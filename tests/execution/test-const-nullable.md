# test-const-nullable

#### server:

```graphql
schema @server @upstream(baseURL: "https://jsonplaceholder.typicode.com") {
  query: Query
}

type User {
  name: String
  age: Int
}

type Query {
  user: User @const(data: {name: "John"})
}
```

#### query:

```graphql
query {
  user {
    age
    name
  }
}
```
