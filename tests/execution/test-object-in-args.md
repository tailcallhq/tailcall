---
error: true
---

# test-object-in-args

```graphql @config
schema @server(port: 8000) {
  query: Query
}
type Query {
  findEmployees(criteria: Nested): [Employee!]!
    @http(
      baseURL: "http://localhost:8081"
      path: "/family/employees"
      query: [{key: "nested", value: "{{.args.criteria}}", skipEmpty: true}]
    )
}

type Employee {
  id: ID!
  name: String!
  age: Int!
  nested: Nested
}

input Nested {
  #  maritalStatus: MaritalStatus
  hasChildren: Boolean
}
```
