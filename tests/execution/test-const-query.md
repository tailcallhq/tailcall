# test-const-query

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
  user: User @const(data: {name: "John", age: 12})
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
