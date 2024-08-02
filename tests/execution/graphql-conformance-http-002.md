# Test complex nested query

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @http(path: "/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  city: String
  birthday: BirthDay!
  friends: [User!]!
}

type BirthDay {
  day: Int!
  month: Int!
  year: Int
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  response:
    status: 200
    body:
      id: 4
      name: Tailcall
      city: Globe
      birthday:
        day: 15
        month: 6
      friends:
        - id: 1
          name: Person 1
          birthday:
            year: null
        - id: 2
          name: Person 2
          birthday:
            year: 2000
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          id
          name
          city
          birthday {
            day
            month
          }
          friends {
            id
            name
            birthday {
              year
            }
          }
        }
      }
```
