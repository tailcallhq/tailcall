---
error: true
---

# Test validation for multiple resolvable directives on field

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
  user1: User
    @expr(body: {name: "{{.value.test}}"})
    @http(url: "http://jsonplaceholder.typicode.com/", query: [{key: "id", value: "{{.value.id}}"}])
  user2: User
    @http(url: "http://jsonplaceholder.typicode.com/", query: [{key: "name", value: "{{.value.name}}"}])
    @expr(body: {name: "{{.args.expr}}"})
  user3: User
    @http(url: "http://jsonplaceholder.typicode.com/", query: [{key: "id", value: "{{.value.address}}"}])
    @graphQL(args: [{key: "id", value: "{{.args.id}}"}], url: "http://upstream/graphql", name: "user")
}
```
