# Test field inputs query

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
  user(id: ID!): User!
    @graphQL(url: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  profilePic(size: Int, width: Int, height: Int): String!
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id name profilePic(size: 100) } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          profilePic: pic_100
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user(id: 4) { id name profilePic(width: 200,height: 100) } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        user:
          id: 4
          name: Tailcall
          profilePic: pic_100_200
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
          profilePic(size: 100)
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        user(id: 4) {
          id
          name
          profilePic(width: 200, height: 100)
        }
      }
```
