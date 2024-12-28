# Resolved by parent

```graphql @schema
schema {
  query: Query
}

type Address {
  street: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User @addField(name: "address", path: ["address", "street"]) {
  address: Address @modify(omit: true)
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      address:
        street: Kulas Light
      id: 1
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { address } }
```
