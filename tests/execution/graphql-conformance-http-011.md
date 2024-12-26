---
skip: true
---

# Test scalars and remote directives.

TODO: Skipped because Tailcall does not parse the scalar type correctly into a string.

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
  nearby(location: Location): Point
    @http(
      url: "http://upstream/nearby"
      query: [{key: "lon", value: "{{.args.location.lon}}"}, {key: "lat", value: "{{.args.location.lat}}"}]
    )
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
    method: GET
    url: http://upstream/nearby?lon=12.43&lat=-53.211
  expectedHits: 1
  response:
    status: 200
    body:
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
