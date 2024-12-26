# Multiple resolvable directives on field

```graphql @schema
schema @server {
  query: Query
}

type User {
  name: String
  id: Int
  address: Address
}

type Address {
  city: String
  street: String
}

type Query {
  user1: User @expr(body: {name: "name expr 1"}) @http(url: "http://jsonplaceholder.typicode.com/users/1")
  user2: User @http(url: "http://jsonplaceholder.typicode.com/users/2") @expr(body: {name: "name expr 2"})
  user3: User
    @http(url: "http://jsonplaceholder.typicode.com/users/3")
    @graphQL(args: [{key: "id", value: "3"}], url: "http://upstream/graphql", name: "user")
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
        city: city request 1
        street: street request 1
      id: 1
      name: from request 1

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/2
  response:
    status: 200
    body:
      address:
        city: city request 2
        street: street request 2
      id: 2
      name: from request 2

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/3
  response:
    status: 200
    body:
      address:
        city: city request 3
      id: 3
      name: name request 3

- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 3) { name address { street city } } }" }'
  response:
    status: 200
    body:
      data:
        user:
          address:
            street: Street from the graphql response
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user1 { name address { street city } }
        user2 { name address { street city } }
        user3 { name address { street city } }
      }
```
