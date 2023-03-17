package tailcall.cli

import tailcall.cli.service.CommandExecutor
import zio.cli._

object CommandDoc {
  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command("compile", Options.file("config").alias("c") ++ Options.directory("output-directory").alias("o").optional)
      .withHelp("Compiles a .yml or .json file into an .orc file").map(CommandADT.Compile.tupled),

    // Schema
    Command("schema", Options.file("blueprint").alias("b")).map(CommandADT.GraphQLSchema)
      .withHelp("Generates a GraphQL schema from a .orc file"),

    // Config
    Command("config", Options.text("key") ++ (Options.text("value").optional))
      .withHelp("Gets and sets CLI configurations").map {
        case (_, Some(value)) => CommandADT.SetRemoteServer(value)
        case (_, None)        => CommandADT.GetRemoteServer
      },

    // Deploy
    Command("deploy", Options.file("orc").alias("o")).withHelp("Deploys an .orc file").map(CommandADT.Deploy),

    // Drop
    Command("drop", CustomOptions.digest("digest").alias("d")).withHelp("Drops a blueprint by its digest")
      .map(CommandADT.Drop(_)),

    // List
    Command(
      "list",
      CustomOptions.integer("index").alias("i").withDefault(0) ++
        CustomOptions.integer("offset").alias("f").withDefault(0)
    ).withHelp("Lists blueprints with pagination (index and offset)").map(CommandADT.GetAll.tupled),

    // Info
    Command("info", CustomOptions.digest("digest").alias("d"))
      .withHelp("Displays information about a blueprint by its digest").map(CommandADT.GetOne)
  )

  val app: CliApp[CommandExecutor, Throwable, CommandADT] = CliApp
    .make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(CommandExecutor.execute(_))
    .summary(HelpDoc.Span.Text("Tailcall CLI"))
}
