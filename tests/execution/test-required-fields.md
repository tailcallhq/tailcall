# Test API

```graphql @config
schema @server @upstream(baseURL: "http://jsonplaceholder.typicode.com") {
  query: Query
}

type Query {
  users: [User] @http(path: "/users")
  posts: [Post] @http(path: "/posts")
  post: Post! @http(path: "/posts/12")
  foo: [Foo]! @http(path: "/foos")
  fooInner: [Foo!] @http(path: "/foos-inner")
}

type Foo {
  id: Int
}

type Post {
  id: Int!
  title: String!
}

type User {
  id: Int!
  name: String!
}
```

```yml @mock
- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/users
  response:
    status: 200
    body:
      - id: 1

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts
  response:
    status: 200
    body:
      - id: 1
        title: "graphql vs rest"
      - id: 2
        title: null

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foos
  response:
    status: 200
    body: null

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/foos-inner
  response:
    status: 200
    body: []

- request:
    method: GET
    url: http://jsonplaceholder.typicode.com/posts/12
  response:
    status: 200
    body: null
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        users {
            name
            id
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        posts {
            title
            id
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        post {
            title
            id
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        foo {
            id
        }
      }

- method: POST
  url: http://localhost:8080/graphql
  body:
    query: >
      query {
        fooInner {
            id
        }
      }
```
