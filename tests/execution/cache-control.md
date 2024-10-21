# Sending requests to verify Cache-Control behavior

```json @config
{
  "server": {
    "headers": {
      "cacheControl": true
    }
  },
  "upstream": {},
  "schema": {
    "query": "Query"
  },
  "types": {
    "Query": {
      "fields": {
        "user": {
          "type": {
            "name": "User"
          },
          "args": {
            "id": {
              "type": {
                "name": "Int"
              }
            }
          },
          "http": {
            "url": "http://jsonplaceholder.typicode.com/users",
            "query": [
              {
                "key": "id",
                "value": "{{.args.id}}"
              }
            ]
          },
          "cache": null
        }
      },
      "cache": null
    },
    "User": {
      "fields": {
        "id": {
          "type": {
            "name": "Int"
          },
          "cache": null
        },
        "name": {
          "type": {
            "name": "String"
          },
          "cache": null
        }
      },
      "cache": null
    }
  }
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=1
    headers:
      test: test
  expectedHits: 3
  response:
    status: 200
    headers:
      Cache-Control: max-age=3600
    body:
      id: 1
      name: foo
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=2
    headers:
      test: test
  response:
    status: 200
    headers:
      Cache-Control: max-age=7200
    body:
      id: 2
      name: bar
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=3
    headers:
      test: test
  expectedHits: 2
  response:
    status: 200
    headers:
      Cache-Control: max-age=7200, private
    body:
      id: 3
      name: foobar
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users?id=4
    headers:
      test: test
  expectedHits: 2
  response:
    status: 200
    headers:
      Cache-Control: no-cache
    body:
      id: 4
      name: barfoo
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) { id } u2: user(id: 2) { id } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) { id } u3: user(id: 3) { id } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u1: user(id: 1) { id } u4: user(id: 4) { id } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { u3: user(id: 3) { id } u4: user(id: 4) { id } }"
```
