
# Basic queries with modify field check

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  user(id: ID!): User! @graphQL(name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String! @modify(name: "newName")
  city: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { city newName } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          city: Globe
          newName: Tailcall
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { city newName id } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          city: Globe
          newName: Tailcall
          id: 4
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id newName city } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          newName: Tailcall
          city: Globe
```

```yml @test
# Positive: basic 1
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          city
          newName
        }
      }
# Positive: basic 2
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          city
          newName
          id
        }
      }
# Positive: basic 2 re ordered
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          newName
          city
        }
      }
# Negative: without selection
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4)
# Negative: non existent fields
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          email_address
        }

# Negative: missing input
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user {
          id
          newName
          city
        }
      }
```
