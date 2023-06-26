package tailcall.server

import tailcall.registry.SchemaRegistry
import zio.cli.{CliApp, Command, Options}
import zio.{Duration, ZIO, ZIOAppArgs}

import java.util.concurrent.TimeUnit

case class GraphQLConfig(
  port: Int = SchemaRegistry.PORT,
  globalResponseTimeout: Duration = Duration(10000, TimeUnit.SECONDS),
  httpCacheSize: Option[Int] = None,
  enableTracing: Boolean = false,
  slowQueryDuration: Option[Duration] = None,
  database: Option[GraphQLConfig.DBConfig] = None,
  persistedQueries: Boolean = false,
)

object GraphQLConfig {
  val default: GraphQLConfig = GraphQLConfig()

  def bootstrap[R, E, A](run: GraphQLConfig => ZIO[R, E, A]): ZIO[R with ZIOAppArgs, Any, Any] =
    ZIOAppArgs.getArgs
      .flatMap(args => CliApp.make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(run(_)).run(args.toList))

  private def command: Command[GraphQLConfig] =
    Command("server", options).withHelp(s"starts the server on port: ${default.port}").map {
      case (port, globalResponseTimeout, httpCacheSize, enableTracing, slowQueryDuration, database, persistedQueries) =>
        GraphQLConfig(
          port,
          globalResponseTimeout,
          httpCacheSize,
          enableTracing,
          slowQueryDuration,
          database,
          persistedQueries,
        )
    }

  private def options =
    CustomOptions.int("port").withDefault(default.port) ++
      CustomOptions.duration("timeout").withDefault(default.globalResponseTimeout) ++
      CustomOptions.int("http-cache").optional.withDefault(default.httpCacheSize) ++
      Options.boolean("tracing").withDefault(default.enableTracing) ++
      CustomOptions.duration("slow-query").optional.withDefault(default.slowQueryDuration) ++
      DBConfig.options ++
      Options.boolean("persisted-queries").withDefault(default.persistedQueries)

  final case class DBConfig(host: String, port: Int, username: Option[String], password: Option[String])
  object DBConfig {
    val options: Options[Option[DBConfig]] = {
      Options.boolean("db").withDefault(false) ++
        Options.text("db-host").withDefault("localhost") ++
        CustomOptions.int("db-port").withDefault(3306) ++
        Options.text("db-username").withDefault("tailcall_main_user").optional ++
        Options.text("db-password").withDefault("tailcall").optional
    }.map { case (enable, host, port, username, password) =>
      if (enable) Some(DBConfig(host, port, username, password)) else None
    }
  }

  private object CustomOptions {
    def duration(name: String): Options[Duration] = Options.integer(name).map(b => Duration.fromMillis(b.toLong))
    def int(name: String): Options[Int]           = Options.integer(name).map(_.toInt)
  }
}
