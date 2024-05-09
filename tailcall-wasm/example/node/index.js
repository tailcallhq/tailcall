const tc = require("@tailcallhq/tailcall-node")

async function run() {
  try {
    await tc.main() // must call
    let schema =
      "schema\n" +
      '  @server(port: 8000, headers: {cors: {allowOrigins: ["*"], allowHeaders: ["*"], allowMethods: [POST, GET, OPTIONS]}})\n' +
      '  @upstream(baseURL: "http://jsonplaceholder.typicode.com", httpCache: true, batch: {delay: 100}) {\n' +
      "  query: Query\n" +
      "}\n" +
      "\n" +
      "type Query {\n" +
      '  posts: [Post] @http(path: "/posts")\n' +
      '  users: [User] @http(path: "/users")\n' +
      '  user(id: Int!): User @http(path: "/users/{{.args.id}}")\n' +
      "}\n" +
      "\n" +
      "type User {\n" +
      "  id: Int!\n" +
      "  name: String!\n" +
      "  username: String!\n" +
      "  email: String!\n" +
      "  phone: String\n" +
      "  website: String\n" +
      "}\n" +
      "\n" +
      "type Post {\n" +
      "  id: Int!\n" +
      "  userId: Int!\n" +
      "  title: String!\n" +
      "  body: String!\n" +
      '  user: User @call(steps: [{query: "user", args: {id: "{{.value.userId}}"}}])\n' +
      "}\n"
    let builder = new tc.TailcallBuilder()
    builder = await builder.with_config("jsonplaceholder.graphql", schema)
    let executor = await builder.build()
    let result = await executor.execute("{posts { id }}")
    console.log("result: " + result)
  } catch (error) {
    console.error("error: " + error)
  }
}

run()
