# test-call-operator

#### server:

```graphql
schema @server @upstream(baseURL: "http://localhost:3000") {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  userWithoutResolver(id: Int!): User
  user(id: Int!): User @http(path: "/users/{{args.id}}")
  userWithGraphQLResolver(id: Int!): User @graphQL(name: "user", args: [{key: "id", value: "{{args.id}}"}])
  userWithGraphQLHeaders(id: Int!): User @graphQL(name: "user", headers: [{key: "id", value: "{{args.id}}"}])
}

type User {
  id: Int!
}

type Post {
  userId: Int!
  withoutResolver: User @call(query: "userWithoutResolver", args: {id: "{{value.userId}}"})
  withoutOperator: User @call(args: {id: "{{value.userId}}"})
  urlMismatchHttp: User @call(query: "user", args: {})
  argumentMismatchGraphQL: User @call(query: "userWithGraphQLResolver", args: {})
  headersMismatchGraphQL: User @call(query: "userWithGraphQLResolver", args: {})
}

#> client-sdl
type Failure @error(message: "No resolver has been found in the schema", trace: ["Query", "userWithoutResolver"])
type Failure
  @error(
    message: "no argument 'id' found"
    trace: ["Post", "argumentMismatchGraphQL", "@call", "userWithGraphQLResolver"]
  )
type Failure
  @error(
    message: "no argument 'id' found"
    trace: ["Post", "headersMismatchGraphQL", "@call", "userWithGraphQLResolver"]
  )
type Failure @error(message: "no argument 'id' found", trace: ["Post", "urlMismatchHttp", "@call", "user"])
type Failure @error(message: "call must have query or mutation", trace: ["Post", "withoutOperator", "@call"])
type Failure @error(message: "userWithoutResolver field has no resolver", trace: ["Post", "withoutResolver", "@call"])
```

#### mock:

```yml
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  response:
    body:
      id: 1
      name: foo
- request:
    url: http://jsonplaceholder.typicode.com/users
    headers:
      id: 1
  response:
    body:
      id: 1
      name: "Leanne Graham http headers"
- request:
    url: http://jsonplaceholder.typicode.com/posts
  response:
    body:
      - id: 1
        userId: 1
- request:
    url: http://jsonplaceholder.typicode.com/users?id=1
  response:
    body:
      id: 1
      name: "Leanne Graham http query"
- request:
    url: http://upstream/graphql
    method: POST
    body: '{ "query": "query { user(id: 1) { name } }" }'
  response:
    body:
      data:
        user:
          name: "Leanne Graham"
- request:
    url: http://upstream/graphql
    method: POST
    body: '{ "query": "query { user { name } }" }'
    headers:
      id: 1
  response:
    body:
      data:
        user:
          name: "Leanne Graham"
- request:
    url: http://jsonplaceholder.typicode.com/users
  response:
    body:
      - id: 1
        name: foo
- request:
    url: http://jsonplaceholder.typicode.com/posts?userId=1
  response:
    body:
      - id: 1
        userId: 1
        title: bar
        body: baz
      - id: 2
        userId: 1
        title: qux
        body: quux
- request:
    url: http://localhost:50051/NewsService/GetAllNews
    method: POST
  response:
    body: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

#### assert:

```yml
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { user { name } } }"
  response:
    body:
      data:
        posts:
          - user:
              name: foo
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { user1 { name } } }"
  response:
    body:
      data:
        posts:
          - user1:
              name: foo
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { userFromValue { name } } }"
  response:
    body:
      data:
        posts:
          - userFromValue:
              name: foo
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { userGraphQLHeaders { name } } }"
    headers:
      id: 1
  response:
    body:
      data:
        posts:
          - userGraphQLHeaders:
              name: "Leanne Graham"
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { userGraphQLHeaders { name } } }"
  response:
    body:
      data:
        posts:
          - userGraphQLHeaders:
              name: "Leanne Graham"
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { userHttpHeaders { name } } }"
  response:
    body:
      data:
        posts:
          - userHttpHeaders:
              name: "Leanne Graham http headers"
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { userHttpQuery { name } } }"
  response:
    body:
      data:
        posts:
          - userHttpQuery:
              name: "Leanne Graham http query"
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { userPosts(id: 1) { title } }"
  response:
    body:
      data:
        userPosts:
          - title: bar
          - title: qux
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { userWithPosts { posts { title } } }"
  response:
    body:
      data:
        userWithPosts:
          posts:
            - title: bar
            - title: qux
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { news { news{ id }} }"
  response:
    body:
      data:
        news:
          news:
            - id: 1
            - id: 2
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { news { news { id } } } }"
  response:
    body:
      data:
        posts:
          - news:
              news:
                - id: 1
                - id: 2
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { posts { newsWithPortArg { news { id } } } }"
  response:
    body:
      data:
        posts:
          - newsWithPortArg:
              news:
                - id: 1
                - id: 2
- request:
    method: POST
    url: http://localhost:8080/graphql
    body:
      query: "query { newsWithPortArg(port: 50051) { news { id } } }"
  response:
    body:
      data:
        newsWithPortArg:
          news:
            - id: 1
            - id: 2
```
