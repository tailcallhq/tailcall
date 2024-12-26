# Add field with composition

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

type Query
  @addField(name: "lat", path: ["user", "address", "geo", "lat"])
  @addField(name: "lng", path: ["user", "address", "geo", "lng"]) {
  user: User @http(url: "http://jsonplaceholder.typicode.com/users/1")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 2
  response:
    status: 200
    body:
      address:
        geo:
          lat: "-37.3159"
          lng: "81.1496"
      id: 1
      name: foo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { lat }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { lng }
```
