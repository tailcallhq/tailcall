# Graphql datasource

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com/") {
  query: Query
}

type Query {
  foo(input: Bar, extra_input: Bar): [Foo]! @http(method: POST, path: "/foo", body: "{{.args.input}}")
}

input Bar {
  buzz: String! @modify(name: "fizz")
  sun: Sun!
}

input Sun {
  moons: String! @modify(name: "planets")
  comments: String @modify(name: "rocks")
}

type Foo {
  id: Int!
  alpha: String!
  beta: String!
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/foo
    body: {"buzz": "test", "sun": {"moons": "test"}}
  response:
    status: 200
    body:
      - id: 1
        alpha: "Foo 1"
        beta: "Foo 1_1"
      - id: 2
        alpha: "Foo 2"
        beta: "Foo 2_1"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        foo(input: {fizz: "test", sun: {planets: "test"}}, extra_input: {fizz: "test", sun: {planets: "test"}}) {
          id
        }
      }
```
