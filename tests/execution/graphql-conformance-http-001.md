# Basic queries with field ordering check

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
  user(id: ID!): User! @http(url: "http://upstream/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  city: String
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 3
  response:
    status: 200
    body:
      id: 4
      name: Tailcall
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
          name
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
          name
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
          name
          city
        }
      }
# Negative: missing fields
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
          name
          city
        }
      }
```
