package tailcall.server

import tailcall.BuildInfo
import tailcall.registry.SchemaRegistry
import zio.cli._
import zio.{Duration, ZIO, ZIOAppArgs}

import java.nio.file.Path

case class GraphQLConfig(
  port: Int = SchemaRegistry.PORT,
  globalResponseTimeout: Int = 10000,
  enableTracing: Boolean = false,
  slowQueryDuration: Option[Int] = None,
  database: Option[GraphQLConfig.DBConfig] = None,
  persistedQueries: Boolean = false,
  allowedHeaders: Set[String] = Set("cookie", "authorization"),
  file: Option[Path] = None,
)

object GraphQLConfig {
  val default: GraphQLConfig = GraphQLConfig()

  def bootstrap[R, E, A](run: GraphQLConfig => ZIO[R, E, A]): ZIO[R with ZIOAppArgs, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args =>
      CliApp.make("tailcall", BuildInfo.version, command.helpDoc.getSpan, command)(run(_)).run(args.toList)
    )

  private def command: Command[GraphQLConfig] =
    Command("server", options).withHelp(s"starts the server on port: ${default.port}").map {
      case (
            port,
            globalResponseTimeout,
            enableTracing,
            slowQueryDuration,
            database,
            persistedQueries,
            allowedHeaders,
            file,
          ) => GraphQLConfig(
          port,
          globalResponseTimeout,
          enableTracing,
          slowQueryDuration,
          database,
          persistedQueries,
          allowedHeaders,
          file,
        )
    }

  private def options = {

    CustomOptions.int("port").withDefault(default.port) ?? "port on which the server starts" ++
      CustomOptions.int("timeout").withDefault(default.globalResponseTimeout) ?? "global timeout in millis" ++
      Options.boolean("tracing")
        .withDefault(default.enableTracing) ?? "enables low-level tracing (affects performance)" ++
      CustomOptions.int("slow-query").optional
        .withDefault(default.slowQueryDuration) ?? "slow-query identifier in millis" ++
      DBConfig.options ++
      Options.boolean("persisted-queries").withDefault(default.persistedQueries) ?? "enable persisted-queries" ++
      Options.text("allowed-headers").map(_.split(",").map(_.trim().toLowerCase()).toSet)
        .withDefault(default.allowedHeaders) ?? "comma separated list of headers" ++
      Options.file("config", Exists.Yes).optional ?? "tailcall configuration file in .yml, .json or .graphql format"
  }

  final case class DBConfig(host: String, port: Int, username: Option[String], password: Option[String])
  object DBConfig {
    val options: Options[Option[DBConfig]] = {
      Options.boolean("db").withDefault(false) ?? "enable database for persistence" ++
        Options.text("db-host").withDefault("localhost") ?? "database hostname" ++
        CustomOptions.int("db-port").withDefault(3306) ?? "database port" ++
        Options.text("db-username").withDefault("tailcall_main_user").optional ?? "database username" ++
        Options.text("db-password").withDefault("tailcall").optional ?? "database password"
    }.map { case (enable, host, port, username, password) =>
      if (enable) Some(DBConfig(host, port, username, password)) else None
    }
  }

  private object CustomOptions {
    def duration(name: String): Options[Duration] = Options.integer(name).map(b => Duration.fromMillis(b.toLong))
    def int(name: String): Options[Int]           = Options.integer(name).map(_.toInt)
  }
}
