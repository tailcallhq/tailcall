# Basic queries with field ordering check

```yaml @config
server:
  port: 8001
  queryValidation: false
  hostname: "0.0.0.0"
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  userCompany(id: Int!): Company @http(url: "http://upstream/users/{{.args.id}}", select: "{{.company}}")
  userDetails(id: Int!): UserDetails
    @http(
      url: "http://upstream/users/{{.args.id}}"
      select: {id: "{{.id}}", city: "{{.address.city}}", phone: "{{.phone}}"}
    )
}

type UserDetails {
  id: Int!
  city: String!
  phone: String!
}

type Company {
  name: String!
  catchPhrase: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/users/1
  expectedHits: 2
  response:
    status: 200
    body:
      id: 1
      company:
        name: FOO
        catchPhrase: BAR
      address:
        city: FIZZ
      phone: BUZZ
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        userCompany(id: 1) {
          name
          catchPhrase
        }
        userDetails(id: 1) {
          city
        }
      }
```
