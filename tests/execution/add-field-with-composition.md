# Add field with composition

#### server:

```graphql
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
          geo:
            lat: "-37.3159"
            lng: "81.1496"
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { lat }
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { lng }
env: {}
```
