---
skip: true
---

Skipped because tailcall does not send the `@log` directive to the remote server.
Moreover it does not correctly format the scalar to string value.

# Test scalars and remote directives

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/graphql", httpCache: 42) {
  query: Query
}

type Query {
  nearby(location: Location): Point @graphQL(name: "nearby", args: [{key: "location", value: "{{.args.location}}"}])
}

type Location {
  lon: Int!
  lat: Int!
}

type Point {
  id: ID!
  name: String!
  location: Location
  createdAt: DateISO
}

scalar DateISO @specifiedBy(url: "https://datatracker.ietf.org/doc/html/rfc3339")
directive @log on FIELD
```

```yml @mock
- request:
    method: POST
    url: http://upstream/graphql
    textBody: '{ "query": "query { nearby(location: {lat: -53.211, lon: 12.43}) { id name createdAt @log } }" }'
  expectedHits: 1
  response:
    status: 200
    body:
      data:
        nearby:
          id: 12
          name: Location 12
          createdAt: "2000-01-01"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
        nearby(location: { lat: -53.211, lon: 12.43 }) {
          id
          name
          createdAt @log
        }
      }
```
