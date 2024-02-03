# Caching parent id object

#### server:

```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query @cache(maxAge: 100) {
  bars: [Bar] @http(path: "/bars")
}

type Foo {
  id: Int
}

type BarId {
  bid: Int
}

type Bar {
  id: [BarId]
  foo: Foo
  flag: Boolean
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://example.com/bars
    body: null
  response:
    status: 200
    body:
      - flag: true
        foo:
          id: 2
        id:
          - bid: 1
      - flag: false
        foo:
          id: 4
        id:
          - bid: 3
      - flag: false
        foo:
          id: 6
        id:
          - bid: 5
      - flag: true
        foo:
          id: 8
        id:
          - bid: 7
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { bars { id { bid } flag foo { id } } }
```
