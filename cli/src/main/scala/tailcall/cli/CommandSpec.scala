package tailcall.cli

import tailcall.cli.CommandADT.Deploy
import tailcall.cli.service.CommandExecutor
import zio.cli._

object CommandSpec {
  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    // Validate Command
    Command("validate", Options.file("file").alias("f")).map(CommandADT.Validate),

    // Deploy Command
    Command("deploy", Options.text("endpoint").alias("e") ++ CustomOptions.digest("digest").alias("d"))
      .map(Deploy.tupled)
  )

  val app: CliApp[CommandExecutor, Throwable, CommandADT] = CliApp
    .make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(CommandExecutor.execute(_))
    .summary(HelpDoc.Span.Text("Tailcall CLI"))
}
