# Caching

#### server:
```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query @cache(maxAge: 100) {
  bars: [Bar] @http(path: "/bars")
}

type Foo {
  id: Int!
}

type Bar {
  id: Int
  foo: Foo
}
```

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://example.com/bars
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
    - foo:
        id: 2
      id: 1
    - foo:
        id: 4
      id: 3
    - foo:
        id: 6
      id: 5
    - foo:
        id: 8
      id: 7
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: query { bars { id foo { id } } }
env: {}
```
