# Js Customize the Response with onResponseBody.

```js @file:test.js
function onResponse(data) {
  const body = JSON.parse(data)
  body.name += " - Changed by JS"
  return JSON.stringify(body)
}
```

```yaml @config
links:
  - src: test.js
    type: Script
```

```graphql @schema
schema {
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
