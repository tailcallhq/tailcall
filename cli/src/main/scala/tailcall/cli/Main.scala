package tailcall.cli

import tailcall.cli.service.{CommandExecutor, Logger}
import tailcall.registry.SchemaRegistryClient
import tailcall.runtime.service._
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault}

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
        SchemaRegistryClient.default
      )
    )
}
