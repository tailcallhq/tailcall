# Tailcall for Nodejs and Browser

It is a simple library that allows you to execute graphql queries on Broswr and Nodejs.

# Setup

## Install (npm)

```shell
npm i @tailcallhq/tailcall-wasm
```

## Local setup

Go to tailcall-wasm dir

```bash
cd tailcall-wasm
```

The [package.json](package.json) file contains the scripts to run the project.

## Nodejs

For test build:

```bash
npm run "dev-node" # compiles in debug mode
```

For release build:

```bash
npm run "build-release-node"  # compiles in release mode with optimizations
```

By default, the output is in the `node` folder.

Now to run the example [index.js](example/node/index.js) file:

```bash
cd example/node && node index.js
```

## Browser

For test build:

```bash
npm run "dev-browser" # compiles in debug mode
```

For release build:

```bash
npm run "build-release-browser"  # compiles in release mode with optimizations
```

By default, the output is in the `browser` folder.

Now to run the example [index.html](example/browser/index.html) file:

Run simple http server in `tailcall-wasm` dir:

```bash
python3 -m http.server 8000
```

you need to pass the schema url as a query parameter as `http://<url>:<port>/?config=<schema-url>`

sample playground: [link](http://0.0.0.0:8000/example/browser/?config=https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql)

# Example

### NodeJS

```javascript
const tc = require("@tailcallhq/tailcall-node")

async function run() {
  try {
    let schema = "https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql"
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
```

### Browser

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    <title>hello-wasm example</title>
  </head>
  <body>
    <div id="content">
      <label for="queryInput"></label><input type="text" id="queryInput" placeholder="Enter your query here" />
      <button id="btn">Run Query</button>
      <p id="result"></p>
    </div>

    <script type="module">
      import init, {TailcallBuilder} from "../../browser/pkg/tailcall_wasm.js"
      await init()

      let executor
      async function setup() {
        try {
          const urlParams = new URLSearchParams(window.location.search)
          let schemaUrl = urlParams.get("config")

          let builder = new TailcallBuilder()
          builder = await builder.with_config("jsonplaceholder.graphql", schemaUrl)
          executor = await builder.build()
          let btn = document.getElementById("btn")
          btn.addEventListener("click", runQuery)
        } catch (error) {
          alert("error: " + error)
        }
      }
      async function runQuery() {
        let query = document.getElementById("queryInput").value
        try {
          document.getElementById("result").textContent = await executor.execute(query)
        } catch (error) {
          console.error("Error executing query: " + error)
          document.getElementById("result").textContent = "Error: " + error
        }
      }
      setup()
    </script>
  </body>
</html>
```
