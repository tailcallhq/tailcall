---
skip: true
---
# Test unions

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  search: [SearchResult!]! @graphQL(name: "search")
}

union SearchResult = Photo | Person

type Person {
  name: String
  age: Int
}

type Photo {
  height: Int
  width: Int
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { search { ... on Person { name } ... on Photo { height } } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        search:
          - __typename: Person
            name: Person
            age: 80
          - __typename: Photo
            height: 100
            width: 200
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        search {
          ... on Person {
            name
          }
          ... on Photo {
            height
          }
        }
      }
```
