# Basic queries with field ordering check

```graphql @config
schema
  @server(port: 8000, hostname: "0.0.0.0", vars: [{key: "id", value: "spam eggs"}])
  @upstream(allowedHeaders: ["Authorization"]) {
  query: Query
}

type Query {
  foo: Foo! @http(url: "http://upstream/foo")
  bar: Bar! @http(url: "http://upstream/foo")
  fizz: Fizz! @http(url: "http://upstream/foo")
  foobar: [String!]! @expr(body: "{{ from_context(\"env.FOOBAR\") | split(\" \") }}")
  token: String! @expr(body: "{{ from_context(\"headers.authorization\") | split(\" \") | .[1] }}")
  var: [String!]! @expr(body: "{{ .vars | .id | split(\" \") }}")
  arg(text: String!): String! @expr(body: "{{ .args.text | split(\" \") | .[0] }}")
}

type Foo {
  bar: [String!]! @expr(body: "{{.value.bar | split(\" \")}}")
}

type Bar {
  bar: [String!]! @expr(body: "{{.value.foo | split(\" \")}}")
}

type Fizz {
  bar: String!
  buzz: Buzz! @expr(body: "{{ .value.bar | split(\" \") | {first: .[0], second: .[1]} }}")
}

type Buzz {
  first: String!
  second: String!
}
```

```json @env
{
  "FOOBAR": "foo bar"
}
```

```yml @mock
- request:
    method: GET
    url: http://upstream/foo
  expectedHits: 3
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

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        bar {
          bar
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        foobar
      }

- method: POST
  url: http://localhost:8080/graphql
  headers:
    authorization: "Bearer JWT_TOKEN"
  body:
    query: |
      {
        token
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        var
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        arg(text: "Hello World")
      }
```
