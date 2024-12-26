---
skip: true
---

# Test unions.

TODO: Skipped because Tailcall does not send the whole query with the **fragments** to the remote server.

```yaml @config
server:
  port: 8001
  hostname: "0.0.0.0"
  queryValidation: false
upstream:
  httpCache: 42
```

```graphql @schema
schema {
  query: Query
}

type Query {
  search: [SearchResult!]! @graphQL(url: "http://upstream/graphql", name: "search")
}

union SearchResult = Photo | Person

type Person {
  name: String
  age: Int
}

type Photo {
  height: Int
  width: Int
  meta: PhotoMeta
}

type PhotoMeta {
  iso: Int
  aparture: Int
  shutter: Int
}

type Page {
  title: String
}
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { search { ... on Person { name } ... on Photo { height meta { iso } } } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        search:
          - __typename: Person
            name: Person
            age: 80
          - __typename: Photo
            height: 100
            width: 200
            meta:
              iso: 200
              aparture: 3
              shutter: 250
- request:
    method: POST
    url: http://upstream/graphql
    textBody: "{ search { ...personFragment ...photoFragment } } fragment personFragment on Person { name } fragment photoFragment on Photo { height ...metaFragment } fragment metaFragment on photo { meta { iso } }"
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        search:
          - __typename: Person
            name: Person
            age: 80
          - __typename: Photo
            height: 100
            width: 200
            meta:
              iso: 200
              aparture: 3
              shutter: 250
```

```yml @test
# Positive: query
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        search {
          ... on Person {
            name
          }
          ... on Photo {
            height
            meta {
              iso
            }
          }
        }
      }
# Positive: fragments
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        search {
          ...personFragment
          ...photoFragment
        }
      }
      fragment personFragment on Person {
        name
      }
      fragment photoFragment on Photo {
        height
        ...metaFragment
      }
      fragment metaFragment on Photo {
        meta {
          iso
        }
      }

# Negative: missing fragment
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        search {
          ...personFragment
          ...photoFragment
        }
      }
      fragment personFragment on Person {
        name
      }
      fragment photoFragment on Photo {
        height
        ...metaFragment
      }
# Negative: unexpected type
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        search {
          ... on Person {
            name
          }
          ... on Page {
            title
          }
          ... on Photo {
            height
          }
        }
      }
```
