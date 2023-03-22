package tailcall.cli

import tailcall.cli.CommandADT.Remote
import tailcall.cli.service.CommandExecutor
import zio.cli._

object CommandDoc {

  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command(
      "check",
      CustomOptions.configFileOption ++ CustomOptions.remoteOption.optional ++ CustomOptions.blueprintOptions
    ).withHelp("Validate a composition spec, display its status when remote is passed.").map {
      case (config, remote, blueprintOptions) => CommandADT.Check(config, remote, blueprintOptions)
    },

    // publish
    Command("publish", CustomOptions.configFileOption ++ CustomOptions.remoteOption)
      .withHelp("Publish the configuration file to the remote environment.").map { case (config, remote) =>
        Remote(remote, Remote.Publish(config))
      },

    // drop
    Command("drop", CustomOptions.digestOption ++ CustomOptions.remoteOption)
      .withHelp("Remove the composition spec from the remote environments using its SHA-256 hash.").map {
        case (digest, remote) => Remote(remote, Remote.Drop(digest))
      },

    // list
    Command(
      "list",
      CustomOptions.remoteOption ++
        CustomOptions.integerOption("offset").withDefault(0) ++
        CustomOptions.integerOption("limit").withDefault(Int.MaxValue)
    ).withHelp("List all published composition specs on the remote address.").map { case (remote, offset, limit) =>
      Remote(remote, Remote.ListAll(offset = offset, limit = limit))
    },

    // info
    Command("show", CustomOptions.digestOption ++ CustomOptions.remoteOption ++ CustomOptions.blueprintOptions)
      .withHelp("Display info for a composition spec using its SHA-256 hash on the remote server.")
      .map { case (digest, remote, blueprintOptions) =>
        Remote(remote, Remote.Show(digest = digest, options = blueprintOptions))
      }
  )

  val app: CliApp[CommandExecutor, Nothing, CommandADT] = CliApp
    .make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(CommandExecutor.execute(_))
    .summary(HelpDoc.Span.Text("Tailcall CLI"))
}
