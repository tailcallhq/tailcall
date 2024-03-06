# Js Hello World

###### sdl error

#### file:test1.js

```js
function onRequest(request) {}
```

#### file:test2.js

```js
function onRequest(request) {}
```

#### server:

```graphql
schema @server @link(type: Script, src: "test1.js") @link(type: Script, src: "test2.js") {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```
