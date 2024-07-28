const tc = require("@tailcallhq/tailcall-node")

async function run() {
  try {
    let schema = "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql"
    let builder = new tc.TailcallBuilder()
    builder = await builder.with_config(schema)
    let executor = await builder.build()
    let result = await executor.execute("{posts { id }}")
    console.log("result: " + result)
  } catch (error) {
    console.error("error: " + error)
  }
}

run()
