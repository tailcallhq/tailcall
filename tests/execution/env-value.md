# Env value

#### server:

```json
{
  "server": {},
  "upstream": {
    "baseURL": "http://jsonplaceholder.typicode.com"
  },
  "schema": {
    "query": "Query"
  },
  "types": {
    "Post": {
      "fields": {
        "body": {
          "type": "String",
          "cache": null
        },
        "id": {
          "type": "Int",
          "cache": null
        },
        "title": {
          "type": "String",
          "cache": null
        },
        "userId": {
          "type": "Int",
          "required": true,
          "cache": null
        }
      },
      "cache": null
    },
    "Query": {
      "fields": {
        "post1": {
          "type": "Post",
          "http": {
            "path": "/posts/{{env.ID}}"
          },
          "cache": null
        },
        "post2": {
          "type": "Post",
          "http": {
            "path": "/posts/{{env.POST_ID}}"
          },
          "cache": null
        },
        "post3": {
          "type": "Post",
          "http": {
            "path": "/posts/{{env.NESTED_POST_ID}}"
          },
          "cache": null
        }
      },
      "cache": null
    }
  }
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/1
    body: null
  response:
    status: 200
    body:
      body: Post 1 body
      id: 1
      title: Post 1
      userId: 1
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/2
    body: null
  response:
    status: 200
    body:
      body: Post 2 body
      id: 2
      title: Post 2
      userId: 2
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/3
    body: null
  response:
    status: 200
    body:
      body: Post 3 body
      id: 3
      title: Post 3
      userId: 3
```

#### env:

```yml
NESTED_POST_ID: "3"
POST_ID: "2"
ID: "1"
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { post1 {id title body userId} post2 {id title body userId} post3 {id title body userId} }
```
