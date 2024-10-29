---
error: true
---

# test objects in args

```graphql @config
schema @server @upstream {
  query: Query
}

type Query {
  findEmployees(criteria: Nested): [Employee!]!
    @http(
      url: "http://localhost:8081/family/employees"
      query: [{key: "nested", value: "{{.args.criteria}}", skipEmpty: true}]
    )
}

type Employee {
  id: ID!
}

input Nested {
  hasChildren: Boolean
}
```
