[![Tailcall Logo](https://raw.githubusercontent.com/tailcallhq/tailcall/main/assets/logo_main.svg)](https://tailcall.run)

Tailcall is an open-source solution for building [high-performance] GraphQL backends.

Please support us by giving the repository a star
![image](https://raw.githubusercontent.com/tailcallhq/tailcall/main/assets/star-our-repo.gif)

[high-performance]: https://github.com/tailcallhq/graphql-benchmarks

[![Tweet](https://img.shields.io/twitter/url/http/shields.io.svg?style=for-the-badge&logo=x&color=black&labelColor=black)](https://twitter.com/intent/tweet?text=%40tailcallhq%20is%20building%20a%20high-performance%20API%20Orchestration%20solution%20over%20%23GraphQL.%0A%0ACheck%20it%20out%20at%3A%0A%F0%9F%94%97%20https%3A%2F%2Ftailcall.run%20%0A%F0%9F%94%97%20https%3A%2F%2Fgithub.com%2Ftailcallhq%2Ftailcall%20%0A%0A&hashtags=api,http,rest,grpc,graphql,nocode,microservice,opensource)
[![Discord](https://img.shields.io/discord/1044859667798568962?style=for-the-badge&cacheSeconds=120&logo=discord)](https://discord.gg/Q2ZExpFCnA)

[![Open Bounties](https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Ftailcallhq%2Fbounties%3Fstatus%3Dopen&style=for-the-badge)](https://console.algora.io/org/tailcallhq/bounties?status=open)
[![Rewarded Bounties](https://img.shields.io/endpoint?url=https%3A%2F%2Fconsole.algora.io%2Fapi%2Fshields%2Ftailcallhq%2Fbounties%3Fstatus%3Dcompleted&style=for-the-badge)](https://console.algora.io/org/tailcallhq/bounties?status=completed)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/tailcallhq/tailcall/ci.yml?style=for-the-badge)](https://github.com/tailcallhq/tailcall/actions)
![GitHub release (by tag)](https://img.shields.io/github/downloads/tailcallhq/tailcall/total?style=for-the-badge)
[![Discord](https://img.shields.io/discord/1044859667798568962?style=for-the-badge&cacheSeconds=60)](https://discord.gg/Q2ZExpFCnA)
[![Codecov](https://img.shields.io/codecov/c/github/tailcallhq/tailcall?style=for-the-badge)](https://app.codecov.io/gh/tailcallhq/tailcall)

## Installation

### NPM

```bash
npm i -g @tailcallhq/tailcall
```

### Yarn

```bash
yarn global add @tailcallhq/tailcall
```

### Home Brew

```bash
brew tap tailcallhq/tailcall
brew install tailcall
```

### Curl

```bash
curl -sSL https://raw.githubusercontent.com/tailcallhq/tailcall/master/install.sh | bash
```

### Docker

```bash
docker pull ghcr.io/tailcallhq/tailcall/tc-server
docker run -p 8080:8080 -p 8081:8081 ghcr.io/tailcallhq/tailcall/tc-server
```

## Get Started

The below file is a standard `.graphQL` file, with a few additions such as `@server` and `@http` directives. So, basically, we specify the GraphQL schema and how to resolve that GraphQL schema in the same file, without having to write any code!

```graphql
schema
  @server(port: 8000, hostname: "0.0.0.0")
  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: 42) {
  query: Query
}

type Query {
  posts: [Post] @http(path: "/posts")
  user(id: Int!): User @http(path: "/users/{{.args.id}}")
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
  id: Int!
  userId: Int!
  title: String!
  body: String!
  user: User @http(path: "/users/{{.value.userId}}")
}
```

Now, run the following command to start the server with the full path to the jsonplaceholder.graphql file that you created above.

```bash
tailcall start ./jsonplaceholder.graphql
```

Head out to [docs] to learn about other powerful tailcall features.

[docs]: https://tailcall.run/docs

### Contributing

Your contributions are invaluable! Kindly go through our [contribution guidelines] if you are a first time contributor.

[contribution guidelines]: https://tailcall.run/developers/

### Support Us

⭐️ Give us a star.

👀 Watch us for updates.

### License

This initiative is protected under the Apache 2.0 License.

<img referrerpolicy="no-referrer-when-downgrade" src="https://static.scarf.sh/a.png?x-pxid=82cc2ee2-ff41-4844-9ae6-c9face103e81" />
