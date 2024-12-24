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
  user(id: ID!): User! @http(url: "http://upstream/user", query: [{key: "id", value: "{{.args.id}}"}])
}

type User {
  id: ID!
  name: String!
  profilePic(size: Int, width: Int, height: Int): String!
    @http(
      url: "http://upstream/pic"
      query: [
        {key: "id", value: "{{.value.id}}"}
        {key: "size", value: "{{.args.size}}"}
        {key: "width", value: "{{.args.width}}"}
        {key: "height", value: "{{.args.height}}"}
      ]
    )
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/user?id=4
  expectedHits: 2
  response:
    status: 200
    body:
      id: 4
      name: Tailcall
- request:
    method: GET
    url: http://upstream/pic?id=4&size=100&width&height
  expectedHits: 1
  response:
    status: 200
    body: profile_pic_size_100
- request:
    method: GET
    url: http://upstream/pic?id=4&size&width=200&height=100
  expectedHits: 1
  response:
    status: 200
    body: profile_pic_200_100
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
