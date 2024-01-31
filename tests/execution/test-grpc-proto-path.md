# test-grpc-proto-path

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type Query {
  news: NewsData
    @grpc(
      service: "NewsService"
      method: "GetAllNews"
      baseURL: "http://localhost:4000"
      protoPath: "tailcall/src/grpcnews.proto"
    )
}

type NewsData {
  news: [News]
}

type News {
  id: Int!
  title: String!
  body: String!
  postImage: String!
}
```
