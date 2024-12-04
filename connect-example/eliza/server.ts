import {fastify} from "fastify"
import {fastifyConnectPlugin} from "@connectrpc/connect-fastify"
import routes from "./connect"

async function main() {
  const server = fastify()
  await server.register(fastifyConnectPlugin, {
    routes,
  })
  server.get("/", (_, reply) => {
    reply.type("text/plain")
    reply.send("Hello World!")
  })
  await server.listen({host: "localhost", port: 8080})
  console.log("server is listening at", server.addresses())
}
// You can remove the main() wrapper if you set type: module in your package.json,
// and update your tsconfig.json with target: es2017 and module: es2022.
void main()
