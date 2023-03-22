package tailcall.cli

import tailcall.cli.CommandADT.Remote
import tailcall.cli.service.CommandExecutor
import tailcall.runtime.ast.Digest
import zio.cli._
import zio.http.URL

import java.nio.file.Path

object CommandDoc {

  private val remoteOption: Options[URL]         = CustomOptions.url("remote").alias("r")
  private val digestOption: Options[Digest]      = CustomOptions.digest("digest")
  private val configFileOption: Options[Path]    = Options.file("config").alias("c")
  private val showSchemaOption: Options[Boolean] = Options.boolean("schema").alias("s").withDefault(false)

  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command("check", configFileOption ++ remoteOption.optional)
      .withHelp("Validate a composition spec, display its status when remote is passed.").map { case (config, remote) =>
        CommandADT.Check(config, remote)
      },

    // publish
    Command("publish", configFileOption ++ remoteOption)
      .withHelp("Publish the configuration file to the remote environment.").map { case (config, remote) =>
        Remote(remote, Remote.Publish(config))
      },

    // drop
    Command("drop", digestOption ++ remoteOption)
      .withHelp("Remove the composition spec from the remote environments using its SHA-256 hash.").map {
        case (digest, remote) => Remote(remote, Remote.Drop(digest))
      },

    // list
    Command(
      "list",
      remoteOption ++
        CustomOptions.integer("offset").withDefault(0) ++
        CustomOptions.integer("limit").withDefault(Int.MaxValue)
    ).withHelp("List all published composition specs on the remote address.").map { case (remote, offset, limit) =>
      Remote(remote, Remote.ListAll(offset = offset, limit = limit))
    },

    // info
    Command(
      "show",
      digestOption ++ remoteOption ++ Options.boolean("blueprint").withDefault(false) ++ showSchemaOption ++ Options
        .boolean("endpoints").withDefault(false)
    ).withHelp("Display info for a composition spec using its SHA-256 hash on the remote server.").map {
      case (digest, remote, showBlueprint, showSchema, showEndpoints) => Remote(
          remote,
          Remote.Show(
            digest = digest,
            showBlueprints = showBlueprint,
            showSchema = showSchema,
            showEndpoints = showEndpoints
          )
        )
    }
  )

  val app: CliApp[CommandExecutor, Nothing, CommandADT] = CliApp
    .make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(CommandExecutor.execute(_))
    .summary(HelpDoc.Span.Text("Tailcall CLI"))
}
