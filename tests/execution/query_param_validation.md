# Query Parameter Validation Test

```graphql @config
schema @server {
  query: Query
}

enum MaritalStatus {
  SINGLE
  MARRIED
  DIVORCED
}

input Nested {
  maritalStatus: MaritalStatus
  hasChildren: Boolean
}

type Employee {
  id: ID!
  name: String!
  maritalStatus: MaritalStatus
  hasChildren: Boolean
}

type Query {
  # This should fail validation since criteria is an object
  findEmployees(criteria: Nested): [Employee!]!
    @http(
      url: "http://localhost:8081/employees"
      query: [
        { key: "nested", value: "{{.args.criteria}}" }
      ]
    )
  
  # This should pass validation since we're using a scalar field
  findEmployeesByStatus(status: MaritalStatus): [Employee!]!
    @http(
      url: "http://localhost:8081/employees"
      query: [
        { key: "status", value: "{{.args.status}}" }
      ]
    )
}
```

```yml @mock
- request:
    method: GET
    url: http://localhost:8081/employees?status=MARRIED
  response:
    status: 200
    body:
      - id: "1"
        name: "John Doe"
        maritalStatus: "MARRIED"
        hasChildren: true
      - id: "2"
        name: "Jane Smith"
        maritalStatus: "MARRIED"
        hasChildren: false
```

```yml @test
# Should fail - object in query param
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        findEmployees(criteria: { maritalStatus: MARRIED, hasChildren: true }) {
          id
          name
        }
      }
  expectedError: "Invalid query parameter type for 'nested'. Expected a Scalar but received an Object."

# Should succeed - scalar in query param  
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        findEmployeesByStatus(status: MARRIED) {
          id
          name
        }
      }
  response:
    data:
      findEmployeesByStatus:
        - id: "1"
          name: "John Doe"
        - id: "2"
          name: "Jane Smith"
```