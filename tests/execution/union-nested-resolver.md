# Field with resolver in one of the possible types of Union

```yaml @config
server:
  port: 8030
```

```graphql @schema
schema {
  query: Query
}

type Query {
  data: [Foo!]!
    @discriminate(field: "object_type")
    @expr(
      body: [
        {id: 1, object_type: "Fizz"}
        {uuid: "hazz-1", object_type: "Hazz"}
        {uuid: "buzz-1", object_type: "Buzz", spam: {identifier: 1}}
        {uuid: "buzz-2", object_type: "Buzz", spam: {identifier: 2}}
        {uuid: "buzz-3", object_type: "Buzz", spam: {identifier: 3}}
      ]
    )
}

union Foo = Fizz | Buzz | Hazz

type Fizz {
  id: Int!
}

type Hazz {
  uuid: String!
}

type Buzz {
  uuid: String!
  spam: Spam
}

type Spam {
  identifier: Int!
  value: String! @http(url: "https://example.com/spam", query: [{key: "identifier", value: "{{.value.identifier}}"}])
}
```

```yml @mock
- request:
    method: GET
    url: https://example.com/spam?identifier=1
  response:
    status: 200
    body: "spam-1"
- request:
    method: GET
    url: https://example.com/spam?identifier=2
  response:
    status: 200
    body: "spam-2"
- request:
    method: GET
    url: https://example.com/spam?identifier=3
  response:
    status: 200
    body: "spam-3"
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      {
          data {
              ... on Buzz {
                  spam {
                      value
                  }
              }
          }
      }
```
