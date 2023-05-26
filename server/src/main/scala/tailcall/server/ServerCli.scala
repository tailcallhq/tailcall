package tailcall.server

import tailcall.registry.SchemaRegistry
import zio.cli.{CliApp, Command, Options}
import zio.{Duration, ZIO, ZIOAppArgs}

import java.util.concurrent.TimeUnit

case class ServerCli(
  port: Int = SchemaRegistry.PORT,
  globalResponseTimeout: Duration = Duration(10000, TimeUnit.SECONDS),
  enableHttpCache: Boolean = false,
  enableTracing: Boolean = false,
  slowQueryDuration: Option[Duration] = None,
)

object ServerCli {
  val default: ServerCli = ServerCli()

  def bootstrap[R, E, A](run: ServerCli => ZIO[R, E, A]): ZIO[R with ZIOAppArgs, Any, Any] =
    ZIOAppArgs.getArgs
      .flatMap(args => CliApp.make("tailcall", "0.0.1", command.helpDoc.getSpan, command)(run(_)).run(args.toList))

  private def command: Command[ServerCli] =
    Command("server", serverOptions).withHelp(s"starts the server on port: ${default.port}").map {
      case (port, globalResponseTimeout, enableHttpCache, enableTracing, slowQueryDuration) =>
        ServerCli(port, globalResponseTimeout, enableHttpCache, enableTracing, slowQueryDuration)
    }

  private def serverOptions =
    CustomOptions.int("port").withDefault(default.port) ++
      CustomOptions.duration("timeout").withDefault(default.globalResponseTimeout) ++
      Options.boolean("http-cache").withDefault(default.enableHttpCache) ++
      Options.boolean("tracing").withDefault(default.enableTracing) ++
      CustomOptions.duration("slow-query").optional.withDefault(default.slowQueryDuration)

  private object CustomOptions {
    def duration(name: String): Options[Duration] = Options.integer(name).map(b => Duration.fromMillis(b.toLong))
    def int(name: String): Options[Int]           = Options.integer(name).map(_.toInt)
  }
}
