# Add field modify

#### server:

```graphql
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
          city: Gwenborough
          street: Kulas Light
          zipcode: 92998-3874
        id: 1
        name: foo
assert:
  - request:
      method: POST
      url: http://localhost:8080/graphql
      headers: {}
      body:
        query: query { user { name street city zipcode } }
env: {}
```
