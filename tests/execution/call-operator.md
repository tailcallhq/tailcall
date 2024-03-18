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

```graphql @server
schema @server(graphiql: true, hostname: "0.0.0.0", port: 8000) @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true) @link(id: "news", src: "news.proto", type: Protobuf) {
  query: Query
}

type News {
  body: String
  id: Int
  postImage: String
  title: String
}

type NewsData {
  news: [News]!
}

type Post {
  body: String
  id: Int
  news: NewsData! @call(query: "news")
  newsWithPortArg: NewsData! @call(query: "news", args: {port: 50051})
  title: String
  user: User @call(query: "user", args: {id: "{{value.userId}}"})
  user1: User @call(query: "user1")
  userFromValue: User @call(query: "userFromValue")
  userGraphQL: User @call(query: "userGraphQL", args: {id: "{{value.userId}}"})
  userGraphQLHeaders: User @call(query: "userGraphQLHeaders", args: {id: "{{value.userId}}"})
  userHttpHeaders: User @call(query: "userHttpHeaders", args: {id: "{{value.userId}}"})
  userHttpQuery: User @call(query: "userHttpQuery", args: {id: "{{value.userId}}"})
  userId: Int!
}

type Query {
  news: NewsData! @grpc(baseURL: "http://localhost:50051", method: "news.NewsService.GetAllNews")
  newsWithPortArg(port: Int!): NewsData! @grpc(baseURL: "http://localhost:{{args.port}}", method: "news.NewsService.GetAllNews")
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{args.id}}")
  user1: User @http(path: "/users/1")
  userFromValue: User @http(path: "/users/{{value.userId}}")
  userGraphQL(id: Int): User @graphQL(args: [{key: "id", value: "{{args.id}}"}], baseURL: "http://upstream/graphql", name: "user")
  userGraphQLHeaders(id: Int!): User @graphQL(baseURL: "http://upstream/graphql", headers: [{key: "id", value: "{{args.id}}"}], name: "user")
  userHttpHeaders(id: ID!): User @http(headers: [{key: "id", value: "{{args.id}}"}], path: "/users")
  userHttpQuery(id: ID!): User @http(path: "/users", query: [{key: "id", value: "{{args.id}}"}])
  userId: Int! @const(data: 2)
  userPosts(id: ID!): [Post] @http(path: "/posts", query: [{key: "userId", value: "{{args.id}}"}])
  userWithPosts: UserWithPosts @http(path: "/users/1")
}

type User {
  email: String!
  id: Int!
  name: String!
  phone: String
  username: String!
  website: String
}

type UserWithPosts {
  id: Int!
  name: String!
  posts: [Post] @call(query: "userPosts", args: {id: "{{value.id}}"})
}
```

```yml @mock
- request:
    url: http://jsonplaceholder.typicode.com/users/1
  expected_hits: 4
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
  expected_hits: 9
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
    body: '{ "query": "query { user { name } }" }'
    headers:
      id: 1
  expected_hits: 2
  response:
    body:
      data:
        user:
          name: "Leanne Graham"
- request:
    url: http://jsonplaceholder.typicode.com/posts?userId=1
  expected_hits: 2
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
  expected_hits: 4
  response:
    body: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

```yml @assert
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
