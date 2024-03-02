# Js Request Response Hello World

#### file:test.js

```js
function onRequest({request}) {
  console.log(request)
  if (request.url.endsWith("/hello")) {
    return {
      response: {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify("hello world"),
      },
    }
  } else if (request.url.endsWith("/hi")) {
    return {
      request: {
        url: "http://localhost:3000/bye",
        method: "GET",
      },
    }
  } else {
    return {request}
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
