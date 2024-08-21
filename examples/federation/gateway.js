import {ApolloServer} from "@apollo/server"
import {startStandaloneServer} from "@apollo/server/standalone"
import {ApolloGateway, IntrospectAndCompose} from "@apollo/gateway"

const gateway = new ApolloGateway({
  supergraphSdl: new IntrospectAndCompose({
    subgraphs: [
      {name: "post", url: "http://localhost:8001/graphql"},
      {name: "user", url: "http://localhost:8002/graphql"},
    ],
  }),
})

const server = new ApolloServer({
  gateway,
  introspection: true,
})

const {url} = await startStandaloneServer(server)
console.log(`🚀  Server ready at ${url}`)
