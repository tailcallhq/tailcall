---
title: CLI
---

Environment variables are key-value pairs stored in our operating systems. Many come by default, and we can also create our own. They are used to store information utilized by our operating system or other programs. For example, the `PATH` variable stores a list of directories the operating system should search when we run a command in the terminal. The `HOME` variable stores the path to our home directory.

These variables are also useful in software development. Configuration values are often stored in environment variables.

## Need for Environment Variables

Applications use multiple external tools, authentication methods, and numerous configurations. Therefore, for proper functioning, our code needs to access these values correctly.

Consider a simple scenario of [JWT authentication](https://jwt.io/). Typically, when signing tokens for our users, we need the following configuration set:

- **Expiry time**: The duration after which the token expires.
- **Secret key**: The key used to encrypt the token.
- **Issuer**: The name of the token issuer, usually the organization's name.

There are broadly two ways to manage this:

1. **Hardcode the values in our code**: \
   This is the simplest but most dangerous and inefficient approach. Hardcoding values in your codebase exposes sensitive information to everyone who works on the code, posing a massive security risk. Also, changing these values requires code modification and application redeployment, which is not ideal.

2. **Store the values in environment variables**: \
   This is the preferred approach. Store sensitive values in the OS of the server running your application. During runtime, your application can access these values from the OS. All programming languages have excellent support for this. This method keeps sensitive information secure and allows value changes without code modifications.

## Environment Variables in Tailcall

With Tailcall, you can seamlessly integrate environment variables into your GraphQL schema. Tailcall supports this through a `env` [Context](context.md) variable. This Context is shared across all operators, allowing you to resolve values in your schema.

Let's take an example. Consider the following schema:

```graphql showLineNumbers
type Query {
  users: [User]! @http(baseUrl: "https://jsonplaceholder.typicode.com", path: "/users")
}
```

Here, we fetch a list of users from the [JSONPlaceholder](https://jsonplaceholder.typicode.com/) API. The `users` field will contain the fetched value at runtime. This works fine, but what if we want to change the API endpoint? We would need to modify the code and redeploy the application, which is cumbersome.

We can address this issue using environment variables. Simply replace the API endpoint with an environment variable, allowing us to change the variable's value without altering our codebase.

```graphql showLineNumbers
type Query {
  users: [User]! @http(baseUrl: "{{env.API_ENDPOINT}}", path: "/users")
}
```

Here, `API_ENDPOINT` is an environment variable that must be set on the device where your server runs. The server picks up this value at startup and makes it available in the `env` Context variable.

This approach allows us to change the API endpoint without modifying our codebase. For instance, we might use different API endpoints for development (`stage-api.example.com`) and production (`api.example.com`) environments.

Note that environment variables are not limited to the `baseUrl` or `@http` operator. Since evaluation is done via a Mustache template, you can use them anywhere in your schema.

Here's another example, using an environment variable in the `headers` of `@grpc`:

```graphql showLineNumbers
type Query {
  users: [User]
    @grpc(
      service: "UserService"
      method: "ListUsers"
      protoPath: "./proto/user_service.proto"
      baseURL: "https://grpc-server.example.com"
      headers: [{key: "X-API-KEY", value: "{{env.API_KEY}}"}]
    )
}
```

## Security Aspects and Best Practices

Environment variables help mitigate many security risks. However, it's important to note that they don't eliminate risks entirely, as the values are still in plain text. While configuration values might not be highly sensitive, secrets can still be compromised.

To ensure your secrets remain secure, consider the following tips:

- **Use a `.env` file**: \
  A common practice is to create a `.env` file in your project's root directory to store all environment variables. This file should not be committed to your version control system and should be added to `.gitignore`. This approach ensures your secrets are not publicly exposed. Additionally, use a `.env.example` file to list all required environment variables for your application, informing other developers of the necessary variables.

  In Tailcall (or elsewhere), you can use this `.env` file by exporting its key-value pairs to your OS.

  For example, if your `.env` file looks like this:

  ```bash
  API_ENDPOINT=https://jsonplaceholder.typicode.com
  ```

  Export it to your OS with:

  ```bash
  export $(cat .env | xargs)
  ```

  On Windows, use:

  ```powershell
  Get-Content .env | Foreach-Object { [System.Environment]::SetEnvironmentVariable($_.Split("=")[0], $_.Split("=")[1], "User") }
  ```

  After this, you can access `API_ENDPOINT` in your codebase.

- **Use Kubernetes Secrets**: \
  If deploying your application with Kubernetes, use its [Secrets](https://kubernetes.io/docs/concepts/configuration/secret/) feature to store environment variables. This ensures your secrets are not publicly exposed and are not hardcoded in your codebase. It also simplifies value changes when needed.

- **Store Secrets Through Cloud Provider GUIs**: \
  When using a cloud provider for deployment, utilize their GUI to store environment variables. These interfaces are usually intuitive and practical, especially for containerized applications that scale automatically.

By following these practices, you can effectively manage and secure your environment variables.
