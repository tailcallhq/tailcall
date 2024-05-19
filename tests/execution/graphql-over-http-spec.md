# GraphQL over HTTP specification

```graphql @config
schema {
  query: Query
}

type User {
  id: Int
  name: String
}

type Query {
  user: User @http(path: "/users/1", baseURL: "http://jsonplaceholder.typicode.com")
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: "*/*"
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json; charset=utf-8
  body:
    query: '{ __type(name: "Run🏃Swim🏊") { name } }'
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
- method: GET
  url: http://localhost:8080/graphql?query=%7B+__typename+%7D
- url: http://localhost:8080/graphql?query=mutation+%7B+__typename+%7D
  headers:
    accept: application/graphql-response+json
- method: POST
  url: http://localhost:8080/graphql
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    notquery: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query:
      obj: ect
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: 0
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: false
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query:
      - array
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    operationName:
      obj: ect
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    operationName: 0
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    operationName: false
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    operationName:
      - array
    query: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    operationName: Query
    query: query Query { __typename }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    operationName: Query
    query: query Query { __typename }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
    variables: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
    variables: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
    operationName: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
    operationName: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
    extensions: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
    extensions: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    variables: string
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    variables: 0
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    variables: false
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    variables:
      - array
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "query Type($name: String!) { __type(name: $name) { name } }"
    variables:
      name: sometype
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "query Type($name: String!) { __type(name: $name) { name } }"
    variables:
      name: sometype
- method: GET
  url: http://localhost:8080/graphql?query=query+Type%28%24name%3A+String%21%29+%7B+__type%28name%3A+%24name%29+%7B+name+%7D+%7D&variables=%7B%22name%22%3A%22sometype%22%7D
  headers:
    accept: application/graphql-response+json
- method: GET
  url: http://localhost:8080/graphql?query=query+Type%28%24name%3A+String%21%29+%7B+__type%28name%3A+%24name%29+%7B+name+%7D+%7D&variables=%7B%22name%22%3A%22sometype%22%7D
  headers:
    accept: application/json
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    extensions: string
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    extensions: 0
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    extensions: false
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    query: "{ __typename }"
    extensions:
      - array
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ __typename }"
    extensions:
      some: value
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ __typename }"
    extensions:
      some: value
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  textBody: '{ "not a JSON'
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  textBody: '{ "not a JSON'
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    qeury: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
  body:
    qeury: "{ __typename }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "{ 8f31403dfe404bccbb0e835f2629c6a7 }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/json
  body:
    query: "query CoerceFailure($id: ID!){ __typename }"
    variables:
      id: null
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ 8f31403dfe404bccbb0e835f2629c6a7 }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ 8f31403dfe404bccbb0e835f2629c6a7 }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "{ 8f31403dfe404bccbb0e835f2629c6a7 }"
- method: POST
  url: http://localhost:8080/graphql
  headers:
    content-type: application/json
    accept: application/graphql-response+json
  body:
    query: "query CoerceFailure($id: ID!){ __typename }"
    variables:
      id: null
```