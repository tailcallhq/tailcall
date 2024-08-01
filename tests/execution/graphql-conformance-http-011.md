# Test scalars and remote directives

```graphql @config
schema
  @server(port: 8001, queryValidation: false, hostname: "0.0.0.0")
  @upstream(baseURL: "http://upstream/", httpCache: 42) {
  query: Query
}

type Query {
  nearby(location: Location): Point
    @http(
      path: "/nearby"
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
