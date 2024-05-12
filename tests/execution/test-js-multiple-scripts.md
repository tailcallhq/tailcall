---
error: true
---

# Js Hello World

```js @file:test1.js
function onRequest(request) {}
```

```js @file:test2.js
function onRequest(request) {}
```

```graphql @config
schema @server @link(type: Script, src: "test1.js") @link(type: Script, src: "test2.js") {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```
