# Add field

#### server:

```graphql
schema {
  query: Query
}

type User @addField(name: "lat", path: ["address", "geo", "lat"]) {
  address: Address
}

type Address {
  geo: Geo
}

type Geo {
  lat: String
}

type Query {
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
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { lat } }
env: {}
```
