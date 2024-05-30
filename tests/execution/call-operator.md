# Test call operator

```protobuf @file:news.proto
syntax = "proto3";

import "google/protobuf/empty.proto";

package news;

message News {
    int32 id = 1;
    string title = 2;
    string body = 3;
    string postImage = 4;
}

service NewsService {
    rpc GetAllNews (google.protobuf.Empty) returns (NewsList) {}
    rpc GetNews (NewsId) returns (News) {}
    rpc GetMultipleNews (MultipleNewsId) returns (NewsList) {}
    rpc DeleteNews (NewsId) returns (google.protobuf.Empty) {}
    rpc EditNews (News) returns (News) {}
    rpc AddNews (News) returns (News) {}
}

message NewsId {
    int32 id = 1;
}

message MultipleNewsId {
    repeated NewsId ids = 1;
}

message NewsList {
    repeated News news = 1;
}
```

```graphql @config
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42)
  @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type Query {
  userId: Int! @expr(body: 2)
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
  userPosts(id: ID!): [Post] @http(path: "/posts", query: [{key: "userId", value: "{{.args.id}}"}])
  user1: User @http(path: "/users/1")
  userFromValue: User @http(path: "/users/{{.value.userId}}")
  userHttpHeaders(id: ID!): User @http(path: "/users", headers: [{key: "id", value: "{{.args.id}}"}])
  userHttpQuery(id: ID!): User @http(path: "/users", query: [{key: "id", value: "{{.args.id}}"}])
  userGraphQL(id: Int): User
    @graphQL(baseURL: "http://upstream/graphql", name: "user", args: [{key: "id", value: "{{.args.id}}"}])
  userGraphQLHeaders(id: Int!): User
    @graphQL(baseURL: "http://upstream/graphql", name: "user", headers: [{key: "id", value: "{{.args.id}}"}])
  userWithPosts: UserWithPosts @http(path: "/users/1")
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:50051")
  newsWithPortArg(port: Int!): NewsData!
    @grpc(method: "news.NewsService.GetAllNews", baseURL: "http://localhost:{{.args.port}}")
}

type NewsData {
  news: [News]!
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}

type UserWithPosts {
  id: Int!
  name: String!
  posts: [Post] @call(steps: [{query: "userPosts", args: {id: "{{.value.id}}"}}])
}

type User {
  id: Int!
  name: String!
  username: String!
  email: String!
  phone: String
  website: String
}

type Post {
  id: Int
  userId: Int!
  title: String
  body: String
  user1: User @call(steps: [{query: "user1"}])
  userFromValue: User @call(steps: [{query: "userFromValue"}])
  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])
  userHttpHeaders: User @call(steps: [{query: "userHttpHeaders", args: {id: "{{.value.userId}}"}}])
  userHttpQuery: User @call(steps: [{query: "userHttpQuery", args: {id: "{{.value.userId}}"}}])
  userGraphQL: User @call(steps: [{query: "userGraphQL", args: {id: "{{.value.userId}}"}}])
  userGraphQLHeaders: User @call(steps: [{query: "userGraphQLHeaders", args: {id: "{{.value.userId}}"}}])
  news: NewsData! @call(steps: [{query: "news"}])
  newsWithPortArg: NewsData! @call(steps: [{query: "news", args: {port: 50051}}])
}
```

```yml @mock
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  expectedHits: 4
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
  expectedHits: 9
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
    textBody: '{ "query": "query { user { name } }" }'
    headers:
      id: 1
  expectedHits: 2
  response:
    body:
      data:
        user:
          name: "Leanne Graham"
- request:
    url: http://jsonplaceholder.typicode.com/posts?userId=1
  expectedHits: 2
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
    url: http://localhost:50051/news.NewsService/GetAllNews
    method: POST
  expectedHits: 4
  response:
    textBody: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { user { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { user1 { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { userFromValue { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { userGraphQLHeaders { name } } }"
    headers:
      id: 1
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { userGraphQLHeaders { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { userHttpHeaders { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { userHttpQuery { name } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { userPosts(id: 1) { title } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { userWithPosts { posts { title } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { news { news{ id }} }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { news { news { id } } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { posts { newsWithPortArg { news { id } } } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { newsWithPortArg(port: 50051) { news { id } } }"
```
