# Caching

#### server:

```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query {
  fieldCache: Type @http(path: "/field-cache") @cache(maxAge: 300)
  fieldCacheList: [Type] @http(path: "/field-cache-list") @cache(maxAge: 300)
  typeCache: TypeCache
}

type Type {
  id: Int
}

type TypeCache @cache(maxAge: 100) {
  a: Type @http(path: "/type-cache-a")
  b: Type @http(path: "/type-cache-b")
  list: [Type] @http(path: "/type-cache-list")
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://example.com/field-cache
  response:
    status: 200
    body:
      id: 1

- request:
    method: GET
    url: http://example.com/field-cache-list
  response:
    status: 200
    body:
      - id: 1
      - id: 2
      - id: 3

- request:
    method: GET
    url: http://example.com/type-cache-a
  response:
    status: 200
    body:
      id: 11

- request:
    method: GET
    url: http://example.com/type-cache-b
  response:
    status: 200
    body:
      id: 21

- request:
    method: GET
    url: http://example.com/type-cache-list
  response:
    status: 200
    body:
      - id: 31
      - id: 32
      - id: 33
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        fieldCache { id }
        fieldCacheList { id }
        typeCache { a { id } , b { id }, list { id } }
      }

# the same request to validate caching
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        fieldCache { id }
        fieldCacheList { id }
        typeCache { a { id } , b { id }, list { id } }
      }
```
