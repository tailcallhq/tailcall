# Basic queries with field ordering check

```graphql @config
schema @server(port: 8000, hostname: "0.0.0.0") {
  query: Query
}

type Query {
  foo: Foo! @http(url: "http://upstream/foo")
  fizz: Fizz! @http(url: "http://upstream/foo")
}

type Foo {
  bar: String!
  bar: [String!]! @expr(body: "{{.value.bar | split(\" \")}}")
}

type Fizz {
  bar: String!
  buzz: Buzz! @expr(body: "{{.value.bar | split(\" \") | {first: .[0], second: .[1]}}}")
}

type Buzz {
  first: String!
  second: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/foo
  expectedHits: 2
  response:
    status: 200
    body:
      bar: "fizz buzz"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        foo {
          bar
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        fizz {
          buzz {
            first
            second
          }
        }
      }
```
