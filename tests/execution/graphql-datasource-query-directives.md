# Graphql datasource with directives in query

Directives in query should be passed as is

```graphql @schema
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @graphQL(url: "http://upstream/graphql", name: "user")
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user @cascade(fields: [\\\"id\\\"]) { id @options(paging: false) name } }" }'
  response:
    status: 200
    body:
      data:
        user:
          id: 123

- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { user @cascade(fields: [\\\"id\\\"]) { id @options(paging: true) name } }" }'
  response:
    status: 200
    body:
      data:
        user:
          id: 123
          name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query($includeName: Boolean!) {
        user @cascade(fields: ["id"]) {
          id @options(paging: $includeName)
          name @include(if: $includeName)
        }
      }
    variables:
      includeName: true

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query($includeName: Boolean!) {
        user @cascade(fields: ["id"]) {
          id @options(paging: $includeName)
          name @include(if: $includeName)
        }
      }
    variables:
      includeName: false
```
