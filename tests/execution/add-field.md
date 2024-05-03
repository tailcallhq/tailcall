# Add field

```graphql @server
schema {
  query: Query
}

type Address {
  geo: Geo
}

type Geo {
  lat: String
}

type Query {
  user: User @http(baseURL: "http://jsonplaceholder.typicode.com", path: "/users/1")
}

type User @addField(name: "lat", path: ["address", "geo", "lat"]) {
  address: Address
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
    query: query { user { lat } }
```
