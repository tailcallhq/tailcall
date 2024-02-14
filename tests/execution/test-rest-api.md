# test-rest-api

#### server:

```graphql
schema {
  query: Query
}

type Address {
  street: String
}

type Query {
  users: [User] @http(path: "/users", baseURL: "http://api.com")
}

type User {
  id: Int
  name: String
  address: Address
}
```

#### rest:

```graphql
query GetUsers @rest(path: "/users") {
  users {
    id
    name
    address {
      street
    }
  }
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://api.com/users
    body: null
  response:
    status: 200
    body:
      - address:
        street: Kulas Light
        id: 1
        name: foo
      - address:
        street: Kulas Dark
        id: 2
        name: bar
```

#### assert:

```yml
- method: GET
  url: http://localhost:8080/api/users
  body:
    query: "query { users { id address } }"
```
