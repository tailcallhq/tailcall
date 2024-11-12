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
schema @link(type: Script, src: "test1.js") @link(type: Script, src: "test2.js") {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:3000/hello")
  hi: String @http(url: "http://localhost:3000/hi")
}
```
