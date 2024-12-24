# List of lists.

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  userGroups: [[User!]!]! @graphQL(url: "http://upstream/graphql", name: "users")
  addUsers(userNames: [[String!]!]!): Boolean
    @graphQL(url: "http://upstream/graphql", name: "addUsers", args: [{key: "userNames", value: "{{.args.userNames}}"}])
}

type User {
  id: ID!
  name: String!
  accountRef: String! @expr(body: "ref-{{.value.id}}-{{.value.name}}")
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { users { id name } }" }'
  response:
    status: 200
    body:
      data:
        users:
          - - id: 1
              name: user-1
            - id: 2
              name: user-2
            - id: 3
              name: user-3
          - - id: 4
              name: user-4
            - id: 5
              name: user-5
            - id: 6
              name: user-6
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { addUsers(userNames: [[\\\"user-1\\\", \\\"user-2\\\"], [\\\"user-3\\\", \\\"user-4\\\"]])  }" }'
  response:
    status: 200
    body:
      data:
        addUsers: true
```

```yml @test
# Positve
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        userGroups {
          id
          name
          accountRef
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        addUsers(userNames: [["user-1", "user-2"], ["user-3", "user-4"]])
      }
# Negative
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        userGroups {
          {
            id
            name
          }
        }
      }
```
