---
error: true
---

# test objects in args

```graphql @schema
schema @server @upstream {
  query: Query
}

type Query {
  invalidArgumentType(criteria: Nested): [Employee!]!
    @http(
      url: "http://localhost:8081/family/employees"
      query: [{key: "nested", value: "{{.args.criteria}}", skipEmpty: true}]
    )
  unknownField(criteria: Nested): [Employee!]!
    @http(
      url: "http://localhost:8081/family/employees"
      query: [{key: "nested", value: "{{.args.criteria.unknown_field}}", skipEmpty: true}]
    )
  unknownArgument(criteria: Nested): [Employee!]!
    @http(
      url: "http://localhost:8081/family/employees"
      query: [{key: "nested", value: "{{.args.criterias}}", skipEmpty: true}]
    )
  invalidArgument(criteria: Nested): [Employee!]!
    @http(url: "http://localhost:8081/family/employees", query: [{key: "nested", value: "{{.args}}", skipEmpty: true}])
  unknownArgumentType(criteria: Criteria): [Employee!]!
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
