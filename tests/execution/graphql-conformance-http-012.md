# Test unions

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  search: [SearchResult!]! @http(path: "/search")
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
    method: GET
    url: http://upstream/search
  expectedHits: 2
  response:
    status: 200
    body:
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
# Disabled because async_graphql::dynamic does not perform validation
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       {
#         search {
#           ...personFragment
#           ...photoFragment
#         }
#       }
#       fragment personFragment on Person {
#         name
#       }
#       fragment photoFragment on Photo {
#         height
#         ...metaFragment
#       }
# # Negative: unexpected type
# Disabled because async_graphql::dynamic does not perform validation
# - method: POST
#   url: http://localhost:8080/graphql
#   body:
#     query: |
#       {
#         search {
#           ... on Person {
#             name
#           }
#           ... on Page {
#             title
#           }
#           ... on Photo {
#             height
#           }
#         }
#       }
```
