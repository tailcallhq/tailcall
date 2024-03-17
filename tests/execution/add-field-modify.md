# Add field modify

```graphql @server
schema @server @upstream {
  query: Query
}

type Address {
  city: String
  street: String
  zipcode: String
}

type Query {
  user: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/1")
}

type User @addField(name: "street", path: ["address", "street"]) @addField(name: "city", path: ["address", "city"]) @addField(name: "zipcode", path: ["address", "zipcode"]) {
  address: Address
  name: String
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
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

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { name street city zipcode } }
```
