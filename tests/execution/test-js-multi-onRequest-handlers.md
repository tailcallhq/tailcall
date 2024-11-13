# Js Request - Multiple onRequest Handlers

```js @file:test1.js
function onRequest({request}) {
  return {request}
}
function foo({request}) {
  return {
    response: {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify("its intercepted foo"),
    },
  }
}
function bar({request}) {
  return {
    response: {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify("its intercepted bar"),
    },
  }
}
```

```graphql @config
schema @link(src: "config.yml", type: Config) @link(type: Script, src: "test1.js") {
  query: Query
}

type Query {
  foo: String @http(url: "http://localhost:3000/foo")
  bar: String @http(url: "http://localhost:3000/bar", onRequest: "bar")
}
```

```yml @file:config.yml
upstream:
  onRequest: "foo"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { foo bar }
```
