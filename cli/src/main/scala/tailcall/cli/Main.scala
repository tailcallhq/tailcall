package tailcall.cli

import tailcall.cli.service.ConfigStore.Key
import tailcall.cli.service.{CommandExecutor, ConfigStore, Logger, RemoteExecutor}
import tailcall.registry.SchemaRegistry
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service._
import zio.http.Client
import zio.rocksdb.RocksDB
import zio.{Scope, ZIO, ZIOAppArgs, ZIOAppDefault, ZLayer}

object Main extends ZIOAppDefault {
  self =>
  override def run: ZIO[Any with ZIOAppArgs with Scope, Any, Any] =
    ZIOAppArgs.getArgs.flatMap(args =>
      CommandDoc.app.run(args.toList).provide(
        CommandExecutor.live,
        Logger.live,
        GraphQLGenerator.live,
        SchemaGenerator.live,
        StepGenerator.live,
        EvaluationRuntime.live,
        RemoteExecutor.live,
        ConfigFileReader.live,
        FileIO.live,
        ZLayer.fromZIO(ConfigStore.getOrDefault(Key.RemoteServer)).flatMap(env => SchemaRegistry.client(env.get)),
        HttpClient.live,
        Client.default,
        ConfigStore.live,
        RocksDB.live(System.getProperty("user.home") + "/tailcall")
      )
    )
}
