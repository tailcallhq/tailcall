# Setting SkipEmpty

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://example.com") {
  query: Query
}

type Query {
  foos: [Foo] @http(path: "/foos")
}

type Foo {
  id: Int!
  tag: String
  bar: Bar
    @http(
      path: "/bar"
      query: [
        # Ignores this query param
        {key: "tagEmpty", value: "{{.value.tag}}", skipEmpty: true}
        {key: "tag", value: "{{.value.tag}}"}
      ]
    )
}

type Bar {
  id: Int
}
```

```yml @mock
- request:
    method: GET
    url: http://example.com/foos
  response:
    status: 200
    body:
      - id: 1
        tag: ABC
      - id: 2
- request:
    method: GET
    url: http://example.com/bar?tagEmpty=ABC&tag=ABC
  response:
    status: 200
    body:
      id: 1
- request:
    method: GET
    url: http://example.com/bar?tag
  response:
    status: 200
    body:
      id: 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { foos { tag id bar { id } } }
```
