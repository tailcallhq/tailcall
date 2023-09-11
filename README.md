[![Tailcall](https://raw.githubusercontent.com/tailcallhq/tailcall/main/assets/logo_main.png)](https://tailcall.run)

Tailcall is an open-source edge and service proxy designed for building high-performance GraphQL applications.

![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/tailcallhq/tailcall/ci.yml?style=for-the-badge)
![GitHub release (by tag)](https://img.shields.io/github/downloads/tailcallhq/tailcall/total?style=for-the-badge)
![Discord](https://img.shields.io/discord/1044859667798568962?style=for-the-badge&cacheSeconds=3600)

## Installation

```bash
brew tap tailcallhq/tailcall
brew install tailcall
```

## Get Started

The below file is a standard `.graphQL` file, with a few additions such as `@server` and `@http` directives. So basically we specify the GraphQL schema and how to resolve that GraphQL schema in the same file, without having to write any code!

[![GraphQL Config Screenshot](https://raw.githubusercontent.com/tailcallhq/tailcall/main/assets/json_placeholder.png)](https://raw.githubusercontent.com/tailcallhq/tailcall/main/assets/jsonplaceholder.graphql)

Now, run the following command to start the server with the full path to the jsonplaceholder.graphql file that you created above.

```bash
tc start ./jsonplaceholder.graphql
```

Head out to [docs] to learn about other powerful tailcall features.

[docs]: https://tailcall.run/docs

### Contributing

Your contributions are invaluable! Kindly fork the repository and propose a pull request.

### Support Us

⭐️ Give us a star.

👀 Watch us for updates.

### License

This initiative is protected under the MIT License.
