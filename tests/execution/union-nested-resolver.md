# Field with resolver in one of the possible types of Union

```graphql @config
schema @server(port: 8030) @upstream(baseURL: "https://example.com/") {
  query: Query
}

type Query {
  data: [Foo!]!
    @discriminate(field: "type")
    @expr(
      body: [
        {id: 1, type: "Fizz"}
        {uuid: "hazz-1", type: "Hazz"}
        {uuid: "buzz-1", type: "Buzz", spam: {identifier: 1}}
        {uuid: "buzz-2", type: "Buzz", spam: {identifier: 2}}
        {uuid: "buzz-3", type: "Buzz", spam: {identifier: 3}}
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
  value: String! @http(path: "/spam", query: [{key: "identifier", value: "{{.value.identifier}}"}])
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
