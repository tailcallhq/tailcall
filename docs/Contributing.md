# Contribution Guidelines

<!-- TOC -->

- [Contribution Guidelines](#contribution-guidelines)
  - [Setting Up the Database](#setting-up-the-database)
  - [Error Handling Strategy](#error-handling-strategy)
    - [Application Errors](#application-errors)
    - [User Errors](#user-errors)
  - [Writing and Including Unit Tests](#writing-and-including-unit-tests)
  - [Additional Contribution Guidelines](#additional-contribution-guidelines)

<!-- TOC -->

<!--
 Link to the GPT conversation to continue later

 https://chat.openai.com/share/9d4446bd-2465-4514-99a5-702b502c3364
 -->

## Setting Up the Database

The Schema Registry for this project requires MySQL 8.0. To set it up:

1. **Install MySQL:** If MySQL 8.0 is not installed, download it from the
   [official website](https://dev.mysql.com/downloads/mysql/).

2. **Execute the** `nuke_registry.sql` **script:** This script is located at
   `/registry/src/main/resources/db/nuke_registry.sql`. When executed, it will
   drop (if they exist) and create the database `tailcall_main_db` and the user
   `tailcall_main_user` with the default password `tailcall`. The user will be
   granted all privileges on the `tailcall_main_db`.

   > ❗️ **Warning:** Running this script will destroy any existing data in the
   > database. Ensure you back up any necessary data before proceeding.

   To run this script, use the command:

   ```
   mysql -u root -p < /registry/src/main/resources/db/nuke_registry.sql
   ```

3. **Run Unit Tests:** After setting up the database, run the unit tests to
   ensure that the database is properly connected. From the sbt console, execute
   the tests using the command:

   ```
   sbt:tailcall> registry/test
   ```

4. **Start the server:** When starting the server, specify the database flag `--db` to use the newly created database.
   From the sbt console, use the following command to start the server:

   ```
   sbt:tailcall> ~ server/reStart --db
   ```

   The server startup should log information about the migrations that were
   executed. These migrations happen automatically every time the server
   restarts.

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

## Writing and Including Unit Tests

When contributing, please adhere to the following guidelines for effective and efficient unit testing:

1. **Mirror your source files:** For each source file, there should be a corresponding unit test file mirroring its structure. For instance, a source file `UserService.scala` should have a corresponding unit test file `UserServiceSpec.scala`.

   While integration tests are not required to mirror source files, their names should indicate their purpose clearly.

2. **Write concise, maintainable tests:** Your tests should be brief and directly related to the behavior of your changes. Avoid extensive bootstrapping and boilerplate. Utilize the `TailcallSpec` for testing with ZIO as it includes safe defaults.

   Remember, good tests also serve as clear, concise documentation. Prioritize readability and understandability in your test writing.

3. **Run all tests before pushing changes:** Use the following command in the sbt console to execute all tests:

   ```
   sbt:tailcall> test
   ```

   Failed tests provide valuable insight into the aspects that need improvement. Use the error messages to guide your debugging.

By ensuring well-written, precise unit tests are included in your pull requests, you contribute to the overall quality of the project and expedite the review process.

## Additional Contribution Guidelines

- Always create a new branch for each feature or bug fix you are working on.
- Write clear, concise, and descriptive commit messages.
- Include tests for new features and bug fixes.
- Make sure your code adheres to the project's code style and formatting guidelines.
- Update the documentation when adding new features or making changes to existing features.
- Submit a pull request for your changes, and ensure that the build and tests pass on the CI server before requesting a
- review.

Thank you for contributing to our project! We appreciate your efforts and look forward to collaborating with you!
