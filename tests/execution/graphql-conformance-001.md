# Basic queries with field ordering check
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
  name: String!
  city: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { city name } }" }'
  expectedHits: 2
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          city: Globe
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { city name id } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          city: Globe
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id name city } }" }'
  expectedHits: 2
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          city: Globe
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          city
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          city
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          city
          name
          id
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        user(id: 4) {
          id
          name
          city
        }
      }

