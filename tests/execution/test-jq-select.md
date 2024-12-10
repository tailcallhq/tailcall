# Basic queries with field ordering check

```graphql @config
schema @server(port: 8001, queryValidation: false, hostname: "0.0.0.0") @upstream(httpCache: 42) {
  query: Query
}

type Query {
  userDetails(id: Int!): UserDetails
    @http(
      url: "http://upstream/users/{{.args.id}}"
      select: {
        id: "{{ .args.id | tostring | explode }}"
        city: "{{ .args.address.city | ascii_upcase }}"
        phone: "{{.args.phone | explode}}"
      }
    )
}

type UserDetails {
  id: [Int!]!
  city: String!
  phone: [Int!]!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/users/1
  expectedHits: 1
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
        userDetails(id: 1) {
          id
          city
          phone
        }
      }
```
