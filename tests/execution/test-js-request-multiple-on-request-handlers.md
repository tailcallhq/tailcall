# Js Request - Multiple onRequest Handlers

```js @file:test1.js
function onRequest({request}) {
  return {request}
}
function life({request}) {
  return {
    response: {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify("its a life."),
    },
  }
}
```

```graphql @server
schema @server @link(type: Script, src: "test1.js") {
  query: Query
}

type Query {
  life: String @http(baseURL: "http://localhost:3000", path: "/life", onRequest: "life")
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { life }
```
