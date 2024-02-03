# test-grpc-nested-optional

###### sdl error

#### server:

```graphql
schema {
  query: Query
}

type Query {
  news: NewsData!
    @grpc(
      service: "news.NewsService"
      method: "GetAllNews"
      baseURL: "http://localhost:4000"
      protoPath: "tests/graphql/errors/proto/news.proto"
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
