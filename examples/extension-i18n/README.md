# Tailcall Extension Example

In this example we showcase the extension capabilities Tailcall supports. We allow the developers to extend Tailcall functionality in the form of custom extensions that enable to hook into the Tailcall runtime. You can utilize extensions using the `@extension` directive. In this project we have examples for two extension scenarios. One for modifying the IR each time, and one to modify a value before returned to the response. See `ExtensionLoader` trait for more information.

## Running

To run the example run the `cargo run -p extension-i18n` command from the root folder of `tailcall` project.

## Example query

```gql
{
  user(id: 1) {
    id
    name
    company {
      catchPhrase
    }
  }
}
```
