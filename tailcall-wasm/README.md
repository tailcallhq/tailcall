# Tailcall for Nodejs and Browser

It is a simple library that allows you to execute graphql queries on Broswr and Nodejs.

# Setup

Go to tailcall-wasm dir

```bash
cd tailcall-wasm
```

The [package.json](package.json) file contains the scripts to run the project.

## Nodejs (locally)

For test build:

```bash
npm run dev node # compiles in debug mode
```

For release build:

```bash
npm run compile node  # compiles in release mode with optimizations
```

By default, the output is in the `node` folder.

Now to run the example [index.js](example/node/index.js) file:

```bash
cd example/node && node index.js
```

## Browser

For test build:

```bash
npm run dev browser # compiles in debug mode
```

For release build:

```bash
npm run compile browser  # compiles in release mode with optimizations
```

By default, the output is in the `browser` folder.

Now to run the example [index.html](example/browser/index.html) file:

Run simple http server in `tailcall-wasm` dir:

```bash
python3 -m http.server 8000
```

To run the example, open the browser and go to `http://localhost:<port>/example/browser/?config=<link to tailcall config>`

For example:

http://localhost:8000/example/browser/?config=https://raw.githubusercontent.com/tailcallhq/tailcall/main/examples/jsonplaceholder.graphql

Make sure to run the http server in `tailcall-wasm` dir because it imports the compiled WASM files from a relative path to `browser` folder.
