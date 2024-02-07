# Js Request Response Hello World

#### file:test.js
```js
// TODO: get rid of this function and do it automatically
function str2ab(str) {
  var buf = new ArrayBuffer(str.length) // 2 bytes for each char
  var bufView = new Uint8Array(buf)
  for (var i = 0, strLen = str.length; i < strLen; i++) {
    bufView[i] = str.charCodeAt(i)
  }
  return buf
}
function onEvent(event) {
  if (event.message.response) {
    return event
  }
  if (event.message.request.method === "GET" && event.message.request.url === "http://localhost:3000/hello") {
    return {
      message: {
        response: {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
          body: str2ab(JSON.stringify("hello world")),
        },
      },
    }
  } else if (event.message.request.method === "GET" && event.message.request.url === "http://localhost:3000/hi") {
    return {
      message: {
        request: {
          url: "http://localhost:3000/bye",
          headers: {},
          method: "GET",
        },
      },
    }
  }
}
```

#### server:

```graphql
schema @server @link(type: Script, src: "test.js") {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://localhost:3000/bye
  response:
    status: 200
    body: hello world
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { hi }
```
