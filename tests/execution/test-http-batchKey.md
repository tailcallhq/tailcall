# Http with args as body

```yaml @config
server:
  port: 8000
upstream:
  batch:
    delay: 10
    maxSize: 1000
```

```graphql @schema
schema {
  query: Query
}

type Query {
  foos: FooResponse @http(url: "http://jsonplaceholder.typicode.com/foo")
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
      url: "http://jsonplaceholder.typicode.com/bar"
      query: [
        #
        {key: "baz", value: "static_value"}
        {key: "barId[]", value: "{{.value.barId}}"}
      ]
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
    url: http://jsonplaceholder.typicode.com/bar?baz=static_value&barId[]=bar_1&barId%5B%5D=bar_2
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
