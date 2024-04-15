---
expect_validation_error: true
---

# Js Hello World

```js @file:test1.js
function onRequest(request) {}
```

```js @file:test2.js
function onRequest(request) {}
```

```graphql @server
schema @link(src: "test1.js", type: Script) @link(src: "test2.js", type: Script) {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```
