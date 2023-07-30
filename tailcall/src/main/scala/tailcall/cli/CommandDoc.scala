package tailcall.cli

import tailcall.cli.CommandADT.Remote
import tailcall.cli.service.CommandExecutor
import tailcall.registry.SchemaRegistry
import zio.cli._

object CommandDoc {

  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command(
      "check",
      CustomOptions.remoteOption.optional ++
        Options.boolean("n-plus-one-queries").alias("npo") ++
        CustomOptions.blueprintOptions,
      Args.file.repeat1,
    ).withHelp("Validate a composition spec, display its status when remote is passed.").map {
      case (remote, nPlusOne, blueprintOptions) -> config => CommandADT
          .Check(config, remote, nPlusOne, blueprintOptions)
    },

    // publish
    Command("publish", CustomOptions.remoteDefaultOption, Args.file.repeat1)
      .withHelp("Publish the configuration file to the remote environment.").map { case (remote, config) =>
        Remote(remote, Remote.Publish(config))
      },

    // drop
    Command("drop", CustomOptions.remoteDefaultOption, CustomArgs.digestArgs)
      .withHelp("Remove the composition spec from the remote environments using its SHA-256 hash.").map {
        case (remote, digest) => Remote(remote, Remote.Drop(digest))
      },

    // list
    Command(
      "list",
      CustomOptions.remoteDefaultOption ++
        CustomOptions.integerOption("offset").withDefault(0) ++
        CustomOptions.integerOption("limit").withDefault(Int.MaxValue),
    ).withHelp("List all published composition specs on the remote address.").map { case (remote, offset, limit) =>
      Remote(remote, Remote.ListAll(offset = offset, limit = limit))
    },

    // info
    Command("show", CustomOptions.remoteDefaultOption ++ CustomOptions.blueprintOptions, CustomArgs.digestArgs)
      .withHelp("Display info for a composition spec using its SHA-256 hash on the remote server.")
      .map { case (remote, blueprintOptions) -> digest =>
        Remote(remote, Remote.Show(digest = digest, options = blueprintOptions))
      },

    // generate
    Command(
      "generate",
      CustomOptions.sourceFormat ++ CustomOptions.targetFormat ++ Options.file("write").optional,
      Args.file.repeat1,
    ).withHelp("Generate a composition spec from a source file.").map {
      case (sourceFormat, targetFormat, write) -> files => CommandADT.Generate(files, sourceFormat, targetFormat, write)
    },
    Command(
      "server",
      CustomOptions.int("port").withDefault(SchemaRegistry.PORT) ?? "port on which the server starts" ++
        CustomOptions.int("timeout").withDefault(10000) ?? "global timeout in millis" ++
        Options.boolean("tracing").withDefault(true) ?? "enables low-level tracing (affects performance)" ++
        CustomOptions.int("slow-query").optional.withDefault(None) ?? "slow-query identifier in millis" ++ {
          // DB configs
          Options.boolean("db").withDefault(false) ?? "enable database for persistence" ++
            Options.text("db-host").withDefault("localhost") ?? "database hostname" ++
            CustomOptions.int("db-port").withDefault(3306) ?? "database port" ++
            Options.text("db-username").withDefault("tailcall_main_user").optional ?? "database username" ++
            Options.text("db-password").withDefault("tailcall").optional ?? "database password"
        }.map { case (enable, host, port, username, password) =>
          if (enable) Some(CommandADT.DBConfig(host, port, username, password)) else None
        } ++
        Options.boolean("persisted-queries").withDefault(false) ?? "enable persisted-queries" ++
        Options.text("allowed-headers").map(_.split(",").map(_.trim().toLowerCase()).toSet)
          .withDefault(Set("cookie", "authorization")) ?? "comma separated list of headers" ++
        Options.file("config", Exists.Yes).optional
          .withDefault(None) ?? "tailcall configuration file in .yml, .json or .graphql format",
    ).withHelp(s"starts the server on the provided port").map {
      case (
            port,
            globalResponseTimeout,
            enableTracing,
            slowQueryDuration,
            database,
            persistedQueries,
            allowedHeaders,
            file,
          ) => CommandADT.ServerStart(
          port,
          globalResponseTimeout,
          enableTracing,
          slowQueryDuration,
          database,
          persistedQueries,
          allowedHeaders,
          file,
        )
    },
  )

  val app: CliApp[CommandExecutor, Nothing, CommandADT] = CliApp
    .make("tailcall", tailcall.BuildInfo.version.replace("v", ""), command.helpDoc.getSpan, command)(
      CommandExecutor.execute(_)
    ).summary(HelpDoc.Span.Text("Tailcall CLI"))
}
