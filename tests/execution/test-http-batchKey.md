# Http with args as body

```graphql @config
schema
  @server(port: 8000)
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", batch: {maxSize: 1000, delay: 10}) {
  query: Query
}

type Query {
  foos: FooResponse @http(path: "/foo")
}

type FooResponse {
  foos: [Foo!]!
}

type Foo {
  id: ID!
  fooName: String!
  barId: String!
  bars: [Bar!]!
    @http(
      path: "/bar"
      query: [{key: "barId[]", value: "{{.value.barId}}"}]
      batchKey: ["bars", "id"]
    )
}

type Bar {
  id: ID!
  barName: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foo
  response:
    status: 200
    body:
      {
        "foos":
          [
            {"id": "foo_1", "fooName": "foo_name_1", "barId": "bar_1"},
            {"id": "foo_2", "fooName": "foo_name_2", "barId": "bar_1"},
            {"id": "foo_3", "fooName": "foo_name_3", "barId": "bar_2"},
          ],
        "meta":
          {"current_page": 1, "next_page": 1, "prev_page": null, "total_pages": 1, "total_count": 3, "per_page": 3},
      }
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar?barId[]=bar_1&barId%5B%5D=bar_2
  response:
    status: 200
    body:
      {
        "bars":
          [
            {"id": "bar_1", "barName": "bar_name_1"},
            {"id": "bar_1", "barName": "bar_name_1"},
            {"id": "bar_2", "barName": "bar_name_2"},
          ],
        "meta":
          {"current_page": 1, "next_page": 1, "prev_page": null, "total_pages": 1, "total_count": 3, "per_page": 3},
      }
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |-
      {
        foos {
          foos {
            id
            fooName
            barId
            bars {
              id
              barName
            }
          }
        }
      }
```
