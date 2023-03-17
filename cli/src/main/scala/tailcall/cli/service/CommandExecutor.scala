package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.cli.service.ConfigStore.Key
import tailcall.registry.SchemaRegistry
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.service.{ConfigFileReader, FileIO, GraphQLGenerator}
import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span.{spans, strong, text}
import zio.json.EncoderOps
import zio.{Duration, ExitCode, UIO, ZIO, ZLayer}

import java.nio.file.Path

trait CommandExecutor {
  def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode]
}

object CommandExecutor {
  final case class Live(
    log: Logger,
    graphQL: GraphQLGenerator,
    remoteExec: RemoteExecutor,
    configReader: ConfigFileReader,
    fileIO: FileIO,
    registry: SchemaRegistry,
    config: ConfigStore
  ) extends CommandExecutor {
    def timed[R, E, A](program: ZIO[R, E, A]): ZIO[R, E, A] =
      for {
        start <- zio.Clock.nanoTime
        a     <- program
        end   <- zio.Clock.nanoTime
        _     <- log {
          val duration = Duration.fromNanos(end - start)
          text(s"\uD83D\uDC4D Completed in ${duration.toMillis} ms.")
        }
      } yield a

    def remoteServer: ZIO[Any, Throwable, String] = config.getOrDefault(Key.RemoteServer)

    override def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Compile(file, output) => for {
              config <- configReader.read(file.toFile)
              blueprint        = config.toBlueprint
              digest           = blueprint.digest
              fileName         = "tc-" + digest.alg + "-" + digest.hex + ".orc"
              outputFile: Path = output.getOrElse(file.getParent).resolve(fileName).toAbsolutePath
              _ <- fileIO.write(outputFile.toFile, blueprint.toJson, FileIO.defaultFlag.withCreate.withTruncateExisting)
              _ <- logSucceed("Compilation completed successfully.")
              _ <- logLabeled("Digest: " -> s"${digest.alg}:${digest.hex}", "Generated File: " -> fileName)
            } yield ()
          case CommandADT.GraphQLSchema(path)   => for {
              blueprint <- fileIO.readJson[Blueprint](path.toFile)
              _         <- logSucceed("GraphQL schema was successfully generated.")
              _         <- logBlueprint(blueprint)
            } yield ()
          case CommandADT.Deploy(path)          => for {
              blueprint <- fileIO.readJson[Blueprint](path.toFile)
              digest    <- registry.add(blueprint)
              server    <- remoteServer
              _         <- logSucceed("Deployment was completed successfully.")
              _         <- logLabeled(
                "Remote Server:" -> server,
                "Digest: "       -> s"${digest.alg}:${digest.hex}",
                "URL: "          -> s"http://${server}/graphQL/${digest.alg}/${digest.hex}"
              )
            } yield ()
          case CommandADT.Drop(digest)          => for {
              _      <- registry.drop(digest)
              server <- remoteServer
              _      <- logSucceed(s"Blueprint with ID '$digest' was dropped successfully.")
              _      <- logLabeled("Remote Server:" -> server, "Digest: " -> s"${digest.alg}:${digest.hex}")
            } yield ()

          case CommandADT.GetAll(index, offset) => for {
              blueprints <- registry.list(index, offset)
              server     <- remoteServer
              _          <- logSucceed("Listing all blueprints.")
              _          <- logLabeled("Remote Server:" -> server, "Total Count: " -> s"${blueprints.length}")
              _          <- ZIO.foreachDiscard(blueprints)(blueprint => log(blueprint.digest.hex))
            } yield ()

          case CommandADT.GetOne(digest) => for {
              info   <- registry.get(digest)
              server <- remoteServer
              _      <- logLabeled(
                "Remote Server:" -> server,
                "Digest: "       -> s"${digest.alg}:${digest.hex}",
                "Status: "       -> (if (info.nonEmpty) "Found" else "Not Found")
              )
              _      <- info match {
                case Some(blueprint) => logBlueprint(blueprint)
                case None            => ZIO.unit
              }
            } yield ()

          case CommandADT.GetRemoteServer =>
            val key = Key.RemoteServer
            for {
              before <- config.get(key)
              _      <- logSucceed("Configuration loaded successfully.")
              _      <- logLabeled("Config Name: " -> key.name, "Value: " -> before.getOrElse(""))
            } yield ()

          case CommandADT.SetRemoteServer(value) =>
            val key = Key.RemoteServer
            for {
              before <- config.get(key)
              _      <- config.set(key, value)
              _      <- logSucceed("Configuration updated successfully.")
              _      <- logLabeled("Config Name: " -> key.name, "Before: " -> before.getOrElse(""), "After: " -> value)
            } yield ()
        }
      }.tapError(log.error(_)).exitCode

    private def logBlueprint(blueprint: Blueprint): UIO[Unit] = { log(text(graphQL.toGraphQL(blueprint).render)) }

    private def logLabeled(labels: (String, String)*): UIO[Unit] = {
      log(HelpDoc.blocks(labels.map { case (key, value) => HelpDoc.p(spans(text(key), strong(value))) }))
    }

    private def logSucceed(message: String): UIO[Unit] = log(strong(message))
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Nothing, ExitCode] =
    ZIO.serviceWithZIO[CommandExecutor](_.dispatch(command))

  type Env = Logger
    with GraphQLGenerator with RemoteExecutor with ConfigFileReader with FileIO with SchemaRegistry with ConfigStore

  def live: ZLayer[Env, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)
}
