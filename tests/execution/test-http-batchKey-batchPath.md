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
  foo_name: String!
  bar_id: String!
  bars: [Bar!]!
    @http(
      path: "/bar"
      query: [{key: "bar_ids[]", value: "{{.value.bar_id}}"}]
      batchKey: "bar_ids[]"
      batchPath: ["bars", "id"]
    )
}

type Bar {
  id: ID!
  bar_name: String!
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
            {"id": "foo_1", "foo_name": "foo_name_1", "bar_id": "bar_1"},
            {"id": "foo_2", "foo_name": "foo_name_2", "bar_id": "bar_1"},
            {"id": "foo_3", "foo_name": "foo_name_3", "bar_id": "bar_2"},
          ],
        "meta":
          {"current_page": 1, "next_page": 1, "prev_page": null, "total_pages": 1, "total_count": 3, "per_page": 3},
      }
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/bar?bar_ids[]=bar_1&bar_ids%5B%5D=bar_2
  response:
    status: 200
    body:
      {
        "bars":
          [
            {"id": "bar_1", "bar_name": "bar_name_1"},
            {"id": "bar_1", "bar_name": "bar_name_1"},
            {"id": "bar_2", "bar_name": "bar_name_2"},
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
            foo_name
            bar_id
            bars {
              id
              bar_name
            }
          }
        }
      }
```
