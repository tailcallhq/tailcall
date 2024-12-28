# Add field modify

```graphql @schema
schema {
  query: Query
}

type User
  @addField(name: "street", path: ["address", "street"])
  @addField(name: "city", path: ["address", "city"])
  @addField(name: "zipcode", path: ["address", "zipcode"]) {
  name: String
  address: Address
}

type Address {
  street: String
  city: String
  zipcode: String
}

type Query {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
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
        city: Gwenborough
        street: Kulas Light
        zipcode: 92998-3874
      id: 1
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name street city zipcode } }
```
