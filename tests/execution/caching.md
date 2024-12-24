# Caching

```yaml @config
upstream:
  batch:
    delay: 1
    maxSize: 1000
```

```graphql @schema
schema {
  query: Query
}

type Query {
  fieldCache: Type @http(url: "http://example.com/field-cache") @cache(maxAge: 30000)
  fieldCacheList: [Type] @http(url: "http://example.com/field-cache-list") @cache(maxAge: 30000)
  typeCache: TypeCache
}

type Type {
  id: Int
}

type TypeCache @cache(maxAge: 1000) {
  a: Type @http(url: "http://example.com/type-cache-a")
  b: Type @http(url: "http://example.com/type-cache-b")
  list: [Type] @http(url: "http://example.com/type-cache-list")
}
```

```yml @mock
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

```yml @test
# the same request to validate caching
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        fieldCache { id }
      }

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
