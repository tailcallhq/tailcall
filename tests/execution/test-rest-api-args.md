# test-rest-api-args

#### server:

```graphql
schema {
  query: Query
}

type Address {
  street: String
}

type Query {
  user(id: Int!, displayName: Boolean!): User
    @http(path: "/user/{{args.id}}/{{args.displayName}}", baseURL: "http://api.com")
  userByName(name: String!): User @http(path: "/userByName/{{args.name}}", baseURL: "http://api.com")
}

type User {
  id: Int
  name: String
  address: Address
}
```

#### rest:

```graphql
query GetUser($id: Int!, $displayName: Boolean!) @rest(path: "/user/$id/$displayName") {
  user(id: $id, displayName: $displayName) {
    id
    name
    address {
      street
    }
  }
}

query GetUserByName($name: String!) @rest(path: "/userByName/$name") {
  userByName(name: $name) {
    id
    name
    address {
      street
    }
  }
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://api.com/user/1/true
    body: null
  response:
    status: 200
    body:
      address:
        street: Kulas Light
      id: 1
      name: foo
- request:
    method: GET
    url: http://api.com/user/1/false
    body: null
  response:
    status: 200
    body:
      address:
        street: Kulas Light
      id: 1
- request:
    method: GET
    url: http://api.com/userByName/foo
    body: null
  response:
    status: 200
    body:
      address:
        street: Kulas Light
      id: 1
      name: foo
```

#### assert:

```yml
- method: GET
  url: http://localhost:8080/api/user/1/true
  body:
    query: "query { user(id: 1, displayName: true) { id name address } }"
- method: GET
  url: http://localhost:8080/api/user/1/false
  body:
    query: "query { user(id: 1, displayName: false) { id name address } }"
- method: GET
  url: http://localhost:8080/api/userByName/foo
  body:
    query: 'query { userByName(name: "foo") { id name address } }'
```
