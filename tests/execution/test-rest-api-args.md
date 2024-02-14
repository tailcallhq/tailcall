# test-rest-api-args

#### server:

```graphql
schema {
  query: Query
}

type Address {
  street: String
}

type Query {
  user(id: Int!): User @http(path: "/users/{{args.id}}", baseURL: "http://api.com")
}

type User {
  id: Int
  name: String
  address: Address
}
```

#### rest:

```graphql
query GetUser($id: Int!) @rest(path: "/users/$id") {
  user(id: $id) {
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
    url: http://api.com/users/1
    body: null
  response:
    status: 200
    body:
      address:
        street: Kulas Light
      id: 1
      name: foo
- request:
    method: GET
    url: http://api.com/users/2
    body: null
  response:
    status: 200
    body:
      address:
        street: Kulas Dark
      id: 2
      name: bar
```

#### assert:

```yml
- method: GET
  url: http://localhost:8080/api/users/1
  body:
    query: "query { user(id: 1) { id address } }"
- method: GET
  url: http://localhost:8080/api/users/2
  body:
    query: "query { user(id: 2) { id address } }"
```
