# Graphql datasource

```graphql @config
schema {
  query: Query
}

enum EnumType {
  INFORMATION
  WARNING
}

type WithOptEnum {
  type: EnumType
}

type Post {
  severity: WithOptEnum!
}

type Query {
  post: Post @graphQL(baseURL: "http://upstream/graphql", name: "post")
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { post { severity { type } } }" }'
  response:
    status: 200
    body:
      data:
        post:
          severity:
            type: null
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { post { severity { type } } }"
```
