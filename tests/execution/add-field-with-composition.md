# Add field with composition


```graphql @server
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
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```


```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  expected_hits: 2
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


```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { lat }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { lng }
```
