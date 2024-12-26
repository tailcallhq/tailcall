# Nested objects

```graphql @schema
schema {
  query: Query
}

type User {
  address: Address
}

type Address {
  street: String
  geo: Geo
}

type Geo {
  lat: String
  lng: String
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
        geo:
          lat: "-37.3159"
      id: 1
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { address { geo { lat } } } }
```
