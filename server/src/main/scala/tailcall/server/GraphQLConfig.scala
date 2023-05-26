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
)

object GraphQLConfig {
  val default: GraphQLConfig = GraphQLConfig()

  def bootstrap[R, E, A](run: GraphQLConfig => ZIO[R, E, A]): ZIO[R with ZIOAppArgs, Any, Any] =
    ZIOAppArgs.getArgs
      .flatMap(args => CliApp.make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(run(_)).run(args.toList))

  private def command: Command[GraphQLConfig] =
    Command("server", serverOptions).withHelp(s"starts the server on port: ${default.port}").map {
      case (port, globalResponseTimeout, httpCacheSize, enableTracing, slowQueryDuration) =>
        GraphQLConfig(port, globalResponseTimeout, httpCacheSize, enableTracing, slowQueryDuration)
    }

  private def serverOptions =
    CustomOptions.int("port").withDefault(default.port) ++
      CustomOptions.duration("timeout").withDefault(default.globalResponseTimeout) ++
      CustomOptions.int("http-cache").optional.withDefault(default.httpCacheSize) ++
      Options.boolean("tracing").withDefault(default.enableTracing) ++
      CustomOptions.duration("slow-query").optional.withDefault(default.slowQueryDuration)

  private object CustomOptions {
    def duration(name: String): Options[Duration] = Options.integer(name).map(b => Duration.fromMillis(b.toLong))
    def int(name: String): Options[Int]           = Options.integer(name).map(_.toInt)
  }
}
