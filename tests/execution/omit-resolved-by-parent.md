# Resolved by parent

#### server:

```graphql
schema {
  query: Query
}

type Address {
  street: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}

type User @addField(name: "address", path: ["address", "street"]) {
  address: Address @modify(omit: true)
}
```

#### assert:

```yml
mock:
  - request:
      method: GET
      url: http://jsonplaceholder.typicode.com/users/1
      headers: {}
      body: null
    response:
      status: 200
      headers: {}
      body:
        address:
          street: Kulas Light
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { address } }
env: {}
```
