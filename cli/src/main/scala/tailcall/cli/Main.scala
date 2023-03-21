package tailcall.cli

import tailcall.cli.service.{CommandExecutor, Logger}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio.http.Client
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault, ZLayer}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args =>
      CommandDoc.app.run(args.toList).provide(
        CommandExecutor.live,
        Logger.live,
        GraphQLGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.live,
        ConfigFileIO.live,
        FileIO.live,
        HttpClient.live,
        Client.default,
        ZLayer.succeed("http://localhost:8080") >>> SchemaRegistry.client
      )
    )
}
