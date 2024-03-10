# Nested objects

####
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

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

####
```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
    body: null
  response:
    status: 200
    body:
      address:
        geo:
          lat: "-37.3159"
      id: 1
      name: foo
```

####
```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { user { address { geo { lat } } } }
```
