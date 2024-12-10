```json @config
{
  "inputs": [
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/posts/1",
        "headers": {
          "Content-Type": "application/json",
          "Accept": "application/json"
        },
        "fieldName": "post"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/users/1",
        "fieldName": "user"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/users",
        "fieldName": "users"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/posts",
        "fieldName": "posts"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/comments",
        "fieldName": "comments"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/comments/1",
        "fieldName": "comment"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/photos",
        "fieldName": "photos"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/photos/1",
        "fieldName": "photo"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/todos",
        "fieldName": "todos"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/todos/1",
        "fieldName": "todo"
      }
    },
    {
      "curl": {
        "src": "{{.env.BASE_URL}}/comments?postId=1",
        "fieldName": "postComments"
      }
    }
  ],
  "preset": {
    "mergeType": 1.0,
    "treeShake": true,
    "inferTypeNames": true
  },
  "output": {
    "path": "./output.graphql"
  },
  "schema": {
    "query": "Query"
  }
}
```

```json @env
{
  "BASE_URL": "http://jsonplaceholder.typicode.com"
}
```
