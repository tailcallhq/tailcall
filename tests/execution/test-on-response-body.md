# Js Customize the Response with onResponseBody.

```js @file:test.js
function onResponse({response}) {
  let body = JSON.parse(response.body)
  body.name = body.name + " - Changed by JS"
  response.body = JSON.stringify(body)
  return {response}
}
```

```graphql @config
schema @server @link(type: Script, src: "test.js") {
  query: Query
}

type Query {
  hello: User! @http(url: "https://jsonplaceholder.typicode.com/users/1", onResponseBody: "onResponse")
}

type User {
  id: Int!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: https://jsonplaceholder.typicode.com/users/1
  response:
    status: 200
    body:
      id: 1
      name: Leanne Graham
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { hello { name } }
```
