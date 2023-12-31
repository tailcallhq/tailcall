---
title: CLI
---

Environment variables are key-value pairs that are stored in our operating systems. Many of them come by default, and we can also create our own. They are used to store information that is used by our operating system or by other programs. For example, the `PATH` variable stores a list of directories that the operating system should look in when we run a command in the terminal. The `HOME` variable stores the path to our home directory.

The same can be used for building softwares aswell. Configuration values are often stored in environmental variables.

## Need of Environmental Variables

Applications use multiple external tools, authentication methods, and numerous configurations. Hence, for proper functioning, we need our code to use these values properly.

Consider a simple scenario of [JWT authentication](https://jwt.io/). Typically, when signing the tokens for our users, we will need the following configuration set:

- **Expiry time**: The time after which the token will expire
- **Secret key**: The secret key that will be used to encrypt the token
- **Issuer**: The name of the issuer of the token. This is generally the organization's name.

To go about this, there are broadly two ways:

1. **Hardcode the values in our code** \
   This is perhaps the easiest approach to solve the problem, but the most dangerous (and inefficient) one. Once you hardcode the values in your codebase, everyone who works on the code will be able to see your sensitive information, which poses a massive security issue. Also, if you want to change the values, you will have to change the code and redeploy the application, which is not ideal.

2. **Store the values in environmental variables** \
   This is the best approach to solve the problem. You can store the sensitive values in the OS of the server running your application, and during the runtime of your application, you can access these values from the OS. All programming languages have excellent support for this. This way, you can keep your sensitive information safe, and also change the values without having to change anything in your code.

## Environmental variables in Tailcall

With Tailcall, you can seamlessly fit in environmental variables into your GraphQL schema. Tailcall supports this by providing a `env` [Context](context.md) variable. This Context is then shared across all the operators, which you will use to resolve values in your schema.

For example, you can do the following:

```graphql showLineNumbers
type Query {
  users: [User]! @http(baseUrl: "{{env.API_ENDPOINT}}", path: "/users")
}
```

Here, the `API_ENDPOINT` is the environmental variable. This value needs to be already set in the device where your server will be running. The value gets picked up by the server when it starts up and is available in the `env` Context variable.

## Security aspects and best practices

Environmental variables help you in mitigating most of the security risks. While this is true, it is also necessary to understand that, it doesn't actually help you in removing the risks completely, since the values are still in plain text. Configurational values might not be as sensitive, but the secrets still do have a chance to be compromised.

To make sure your secrets stay safe and secure all through, here are some tips and tricks that you might consider using for your application:

- **Use a `.env` file** \
  This is a common practice that is followed by most developers. You can create a `.env` file in the root directory of your project and store all your environmental variables in it. This file should not be committed to your version control system, and should be added to the `.gitignore` file. This way, you can make sure that your secrets are not exposed to the public. Additionally, you can also use a `.env.example` file to list out all the environmental variables that are required for your application to run. This way, you can make sure that all the developers who work on your project are aware of the variables that are required for the application to run.

- **Use secrets in K8s** \
  If you are using Kubernetes to deploy your application, you can use the [Secrets](https://kubernetes.io/docs/concepts/configuration/secret/) feature to store your environmental variables. This way, you can make sure that your secrets are not exposed to the public, and also, you can make sure that the values are not hardcoded in your codebase. Not to mention, it becomes very easy to change the values when required.

- **Store secrets through the GUI** \
  If you are using a cloud provider to deploy your application, you can use the GUI provided by the cloud provider to store your environmental variables. They generally provide an intuitive UI, and furthermore, if your application is containerized and scales up and down automatically, this becomes the default go-to option.

- **Using external services** \
  There are many providers out there, mostly open-sourced softwares, that lift off the burden of security from your shoulders and provide you with a secure way to store your secrets. You can use these services to store your secrets and then access them in your application. Some of the popular ones are [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/), [Hashicorp Vault](https://www.vaultproject.io/), [Keywhiz](https://square.github.io/keywhiz/), etc.
