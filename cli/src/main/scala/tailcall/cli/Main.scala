package tailcall.cli

import tailcall.cli.service.{CommandExecutor, Logger, RemoteExecutor}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio.http.Client
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args =>
      CommandSpec.app.run(args.toList).provide(
        CommandExecutor.live,
        Logger.live,
        GraphQLGenerator.live,
        SchemaGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.live,
        RemoteExecutor.live,
        ConfigFileReader.live,
        FileIO.live,
        SchemaRegistry.client("locahost:8080"),
        HttpClient.live,
        Client.default
      )
    )
}
