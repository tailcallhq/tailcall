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
schema @server @upstream(baseURL: "http://localhost:3000", onRequest: "foo") @link(type: Script, src: "test1.js") {
  query: Query
}

type Query {
  foo: String @http(baseURL: "http://localhost:3000", path: "/foo")
  bar: String @http(baseURL: "http://localhost:3000", path: "/bar", onRequest: "bar")
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { foo bar }
```
