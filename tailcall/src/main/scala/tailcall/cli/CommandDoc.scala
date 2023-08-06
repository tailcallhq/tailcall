package tailcall.cli

import tailcall.cli.service.CommandExecutor
import tailcall.registry.SchemaRegistry
import zio.cli._

object CommandDoc {

  val command: Command[CommandADT] = Command("tc", Options.none).subcommands(
    Command(
      "check",
      Options.boolean("n-plus-one-queries").alias("npo") ++
        CustomOptions.blueprintOptions,
      Args.file.repeat1,
    ).withHelp("Validate a composition spec").map { case (nPlusOne, blueprintOptions) -> config =>
      CommandADT.Check(config = config, nPlusOne = nPlusOne, options = blueprintOptions)
    },

    // generate
    Command(
      "generate",
      CustomOptions.sourceFormat ++ CustomOptions.targetFormat ++ Options.file("write").optional,
      Args.file.repeat1,
    ).withHelp("Generate a composition spec from a source file.").map {
      case (sourceFormat, targetFormat, write) -> files => CommandADT.Generate(files, sourceFormat, targetFormat, write)
    },

    // start
    Command(
      "start",
      CustomOptions.int("port").withDefault(SchemaRegistry.PORT) ?? "port on which the server starts" ++
        CustomOptions.int("timeout").withDefault(10000) ?? "global timeout in millis" ++
        Options.boolean("tracing") ?? "enables low-level tracing (affects performance)" ++
        CustomOptions.int("slow-query").optional.withDefault(None) ?? "slow-query identifier in millis" ++ {
          // DB configs
          Options.boolean("db") ?? "enable database for persistence" ++
            Options.text("db-host").withDefault("localhost") ?? "database hostname" ++
            CustomOptions.int("db-port").withDefault(3306) ?? "database port" ++
            Options.text("db-username").optional.withDefault(Option("tailcall_main_user")) ?? "database username" ++
            Options.text("db-password").optional.withDefault(Option("tailcall")) ?? "database password"
        }.map { case (enable, host, port, username, password) =>
          if (enable) Some(CommandADT.DBConfig(host, port, username, password)) else None
        } ++
        Options.boolean("persisted-queries") ?? "enable persisted-queries" ++
        Options.text("allowed-headers").map(_.split(",").map(_.trim().toLowerCase()).toSet)
          .withDefault(Set("cookie", "authorization")) ?? "comma separated list of headers",
      Args.file(Exists.Yes).atMost(1),
    ).withHelp(
      s"starts the server on the provided port and optionally expects a tailcall configuration file in .yml, .json or .graphql format"
    ).map {
      case (
            (port, globalResponseTimeout, enableTracing, slowQueryDuration, database, persistedQueries, allowedHeaders),
            file,
          ) => CommandADT.ServerStart(
          port,
          globalResponseTimeout,
          enableTracing,
          slowQueryDuration,
          database,
          persistedQueries,
          allowedHeaders,
          file.headOption,
        )
    },
  )

  val app: CliApp[CommandExecutor, Nothing, CommandADT] = CliApp
    .make("tailcall", tailcall.BuildInfo.version.replace("v", ""), command.helpDoc.getSpan, command)(
      CommandExecutor.execute
    ).summary(HelpDoc.Span.Text("Tailcall CLI"))
}
