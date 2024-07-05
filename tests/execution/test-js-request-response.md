# Js Request Response Hello World

```js @file:test.js
function onRequest({request}) {
  if (request.uri.path.endsWith("/hello")) {
    return {
      response: {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify("hello world"),
      },
    }
  } else if (request.uri.path.endsWith("/hi")) {
    request.uri.path = "/bye"
    console.log({request})
    return {request}
  } else {
    return {request}
  }
}
```

```graphql @config
schema @server @upstream(onRequest: "onRequest") @link(type: Script, src: "test.js") {
  query: Query
}

type Query {
  hello: String @http(baseURL: "http://localhost:3000", path: "/hello")
  hi: String @http(baseURL: "http://localhost:3000", path: "/hi")
}
```

```yml @mock
- request:
    method: GET
    url: http://localhost:3000/bye
  response:
    status: 200
    body: bye world
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { hello hi }
```
