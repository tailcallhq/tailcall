# Connect RPC Examples

This directory contains examples demonstrating how to use Connect RPC with Tailcall. There are two examples available:

- News Service: A simple news management service
- Eliza Service: A chatbot service

## Prerequisites

- Node.js (v14 or later)
- npm
- Tailcall CLI

## Running the Examples

### 1. Choose an Example

Navigate to either the news or eliza example directory:

```bash
cd connect-example/news
# OR
cd connect-example/eliza
```

### 2. Install Dependencies

Install the required Node.js dependencies:

```bash
npm install
```

### 3. Start the Connect RPC Server

Start the example server:

```bash
npm start
```

This will start the Connect RPC server on port 8080.

### 4. Start the GraphQL Server

In a new terminal window, start the Tailcall GraphQL server using the generated configuration:

```bash
tailcall start ./output.graphql
```

The GraphQL server will start and be available at `http://localhost:8000/graphql`.

## Testing the Service

- Use the provided curl examples in `curl-examples.md` to test the Connect RPC endpoints
- Use GraphQL Playground at `https://tailcall.run/playground/?u=http://127.0.0.1:8000/graphql&utm_source=tailcall-debug&utm_medium=server` to test the GraphQL API

## Configuration

The GraphQL schema and resolvers are generated from the `simple-json.json` configuration file. You can modify this file to customize the service behavior.
