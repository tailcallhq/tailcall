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
schema {
  query: Query
}

type Query {
  hello: String @http(url: "http://localhost:3000/hello")
  hi: String @http(url: "http://localhost:3000/hi")
}
```

```yml @file:config.yml
schema: {}
upstream:
  onRequest: "foo"
links:
  - type: Script
    src: "test1.js"
links:
  - type: Script
    src: "test2.js"
```
