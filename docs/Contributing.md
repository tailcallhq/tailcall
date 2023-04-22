Contribution Guidelines
---

<!-- TOC -->

* [Contribution Guidelines](#contribution-guidelines)
* [Error Handling Strategy](#error-handling-strategy)
  * [Application Errors](#application-errors)
  * [User Errors](#user-errors)
* [Additional Contribution Guidelines](#additional-contribution-guidelines)

<!-- TOC -->

## Error Handling Strategy

We classify errors into two main categories:

**Application Errors:** These errors occur during the execution of the program due to logical issues. Ideally, they
should
never happen. When they do, you can throw them as a RuntimeException, or catch and log them accordingly.
User Errors: These errors occur because of invalid input or configuration. They should be printed on the CLI or returned
by the server, along with helpful information on how the user can fix the error.
To maintain consistency and improve the user experience, please follow the error handling strategy outlined below.

### Application Errors

When an application error occurs, throw a RuntimeException with a clear and concise error message.
If you want to log the error, catch the RuntimeException, log the error message, and re-throw the exception to halt the
program's execution.

```scala
def program = {
  // Your application's core logic here
  ???
}
program.catchAll {
  case e: RuntimeException => ZIO.logError("Application error: " + e.getMessage)
  case e => ZIO.fail(e)
}
```

### User Errors

Create custom exception classes extending the `ValidationError` class to represent specific user error scenarios.

```scala
class InvalidInputException(message: String) extends ValidationError(message)

class InvalidConfigurationException(message: String) extends ValidationError(message)
```

When a user error occurs, throw the corresponding custom exception with a helpful error message, including instructions
on how to fix the error.

By adhering to this error handling strategy, we can maintain a consistent approach to handling both application and user
errors throughout the project, ensuring a better user experience and easier debugging.

## Additional Contribution Guidelines

- Always create a new branch for each feature or bug fix you are working on.
- Write clear, concise, and descriptive commit messages.
- Include tests for new features and bug fixes.
- Make sure your code adheres to the project's code style and formatting guidelines.
- Update the documentation when adding new features or making changes to existing features.
- Submit a pull request for your changes, and ensure that the build and tests pass on the CI server before requesting a
- review.

Thank you for contributing to our project! We appreciate your efforts and look forward to collaborating with you!