---
title: Watch Mode
---

Developers often find themselves in situations where they need to run a server in watch mode to streamline the development process. This guide will introduce you to [entr], a versatile file-watcher tool, and demonstrate how to run your server in watch mode with it. We'll also touch on the installation process and suggest some best practices to optimize your workflow.

[entr]: https://eradman.com/entrproject/

## Use case

Running a server in watch mode offers several key benefits:

- `Real-time Feedback` : Watch mode ensures that your server stays up-to-date with your code changes. It immediately reflects those changes, providing you with real-time feedback during development.
- `Efficiency` : Manually restarting the server each time you modify code can be tedious and time-consuming. Watch mode automates this process, making development more efficient.
- `Debugging` : It helps you quickly identify and fix issues as they arise, reducing the debugging time. When your server automatically restarts upon code changes, you catch errors sooner.

## Using `entr`

`entr` is a powerful file-watching utility that makes running a server in watch mode a breeze. Let's go through the steps for the installation process for different operating system :

### Installation

#### Homebrew

1. Open the Terminal, which you can find in the "Utilities" folder within the "Applications" folder.

2. Install Homebrew if you haven't already. Run the following command in your Terminal:

   ```graphql
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
   ```

3. Once Homebrew is installed, you can install `entr` by running the following command:

   ```graphql
   brew install entr
   ```

4. To verify the installation, run:

   ```graphql
   entr --version
   ```

If the installation is done correctly it will shown the latest version of the `entr`

#### Windows Subsystem

1. Install Windows Subsystem for Linux (WSL) on your Windows machine by following Microsoft's official documentation.

2. After setting up WSL, open the Linux terminal by running:

   ```graphql
       wsl -d <DistributionName>
   ```

   Replace `<DistributionName>` with the name of the Linux distribution that you have installed.

3. Install entr within the Linux terminal using the package manager of your chosen Linux distribution. For example, on Ubuntu, you can use:

   ```graphql
   sudo apt update
   sudo apt install entr
   ```

4. Verify the installation by running:

   ```graphql
   entr --version
   ```

If the installation is done correctly it will shown the latest version of the `entr`

#### apt-get

1. On Linux, you can install `entr` using your distribution's package manager. For example, on Ubuntu, use:

   ```graphql
   sudo apt update
   sudo apt install entr
   ```

2. To verify the installation, run:

   ```graphql
   entr --version
   ```

If the installation is done correctly it will shown the latest version of the `entr`

### Watch Mode

To run your server in watch mode using `entr`, you'll utilize the `ls` command to list the files you want to monitor. The general syntax is as follows:

```graphql
ls *.graphql | entr -r tailcall start ./jsonplaceholder.graphql
```

This command uses `entr` to continuously monitor the `jsonplaceholder.graphql` file and when it changes, It runs the `tailcall start` command with the file as an argument

The above command is described in detail below :

1. `ls *.graphql` : This part of the code lists the file or files you want to monitor for changes. In this case, it lists the file named "jsonplaceholder.graphql" within the "examples" directory.

2. `|` : The pipe symbol ('|') is used to take the output of the preceding command (the file listing) and feed it as input to the following command (entr).

3. `entr -r tc start ./jsonplaceholder.graphql` : This is the command that will be executed whenever the file "jsonplaceholder.graphql" changes.

- `entr` is a command-line tool for running arbitrary commands whenever files change. It monitors the files specified in the previous command (`ls ./jsonplaceholder.graphql`)

- `r` : This flag tells entr to continue running the command even if it encounters errors (it runs the command repeatedly).

- `tc start ./jsonplaceholder.graphql` : This is the command to run when changes are detected. It is executing a command `tc start` with the file path
  `./jsonplaceholder.graphql` as an argument

## Some Best Practices

To make the most of running a server in watch mode with `entr`, consider the following best practices:

1. **Selective File Watching**: Be selective about which files you monitor with `entr`. Watching unnecessary files can lead to increased CPU and memory usage. Focus on the essential files related to your project.

2. **Organize Your Project**: Maintain a well-organized project structure to make it easier to identify which files need monitoring.

3. **Clear Output**: Clear the terminal output before running entr to have a clean workspace.

4. **Version Control**: Ensure that your project is under version control (e.g., Git) to track changes and easily revert if necessary.

5. **Update `entr`**: Kepp `entr` up to date with the latest version to benefit from bug fixes and improvements.

By following these best practices and using `entr` effectively, you can significantly improve your development workflow. Experiment with `entr`, adapt it to your project's specific requirements, and enjoy a smoother and more efficient development process. Happy coding!
