package tailcall.cli

import tailcall.cli.service.CommandExecutor
import zio.cli._

object CommandSpec {
  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command("compile", Options.file("config").alias("c") ++ Options.directory("output-directory").alias("o").optional)
      .withHelp("Compiles a .yml or .json file into an .orc file").map(CommandADT.Compile.tupled),

    // Schema
    Command("schema", Options.file("blueprint").alias("b")).map(CommandADT.GraphQLSchema)
      .withHelp("Generates a GraphQL schema from a .orc file")
  )

  val app: CliApp[CommandExecutor, Throwable, CommandADT] = CliApp
    .make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(CommandExecutor.execute(_))
    .summary(HelpDoc.Span.Text("Tailcall CLI"))
}
