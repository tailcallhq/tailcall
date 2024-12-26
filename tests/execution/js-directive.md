# Js Request Response Hello World

```js @file:test.js
function onRequest({request}) {
  return {request}
}

function name(val) {
  let json = JSON.parse(val)
  return JSON.stringify(json.name.toUpperCase())
}
```

```yaml @config
links:
  - type: Script
    src: test.js
```

```graphql @schema
schema {
  query: Query
}

type Query {
  hello: User! @http(url: "http://jsonplaceholder.typicode.com/users/1")
}

type User {
  id: Int!
  name: String! @js(name: "name")
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users/1
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
