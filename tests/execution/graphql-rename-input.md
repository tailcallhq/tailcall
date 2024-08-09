# Graphql datasource

```graphql @config
schema @server(port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com/") {
  query: Query
}

type Query {
  foo(input: Bar!, extra_input: Spam!): String!
    @http(
      method: POST
      path: "/foo"
      body: "{{.args.input}}"
      query: [{key: "spamIds[]", value: "{{.args.extra_input.id}}"}]
    )
}

input Bar {
  info: String! @modify(name: "bar")
  foo: [Foo!] @modify(name: "buzz")
}

input Foo {
  info: String! @modify(name: "foo")
  bar: Bar @modify(name: "fizz")
}

input Spam {
  id: ID! @modify(name: "identifier")
}
```

```yml @mock
- request:
    method: POST
    url: http://jsonplaceholder.typicode.com/foo?spamIds[]=1
    body:
      {"foo": [{"bar": {"info": "bar_1"}, "info": "foo_1"}, {"bar": {"info": "bar_1"}, "info": "foo_1"}], "info": "bar"}
  response:
    status: 200
    body: Hello from buzz
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        foo(
          input: {buzz: [{fizz: {bar: "bar_1"}, foo: "foo_1"}, {fizz: {bar: "bar_1"}, foo: "foo_1"}], bar: "bar"}
          extra_input: {identifier: 1}
        )
      }
```
