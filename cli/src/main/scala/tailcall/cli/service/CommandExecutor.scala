package tailcall.cli.service

import tailcall.cli.CommandADT
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
    registry: SchemaRegistry
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
              _         <- logSucceed("Deployment was completed successfully.")
              _         <- logLabeled(
                "Remote Server:" -> "http://localhost:8080",
                "Digest: "       -> s"${digest.alg}:${digest.hex}",
                "URL: "          -> s"http://localhost:8080/graphQL/${digest.alg}/${digest.hex}"
              )
            } yield ()
          case CommandADT.Drop(digest)          => for {
              _ <- registry.drop(digest)
              _ <- logSucceed(s"Blueprint with ID '$digest' was dropped successfully.")
              _ <- logLabeled("Remote Server:" -> "http://localhost:8080", "Digest: " -> s"${digest.alg}:${digest.hex}")
            } yield ()

          case CommandADT.List(index, offset) => for {
              blueprints <- registry.list(index, offset)
              _          <- logSucceed("Listing all blueprints.")
              _ <- logLabeled("Remote Server:" -> "http://localhost:8080", "Total Count: " -> s"${blueprints.length}")
              _ <- ZIO.foreachDiscard(blueprints)(blueprint => log(blueprint.digest.hex))
            } yield ()

          case CommandADT.Info(digest) => for {
              info <- registry.get(digest)
              _    <- logLabeled(
                "Remote Server:" -> "http://localhost:8080",
                "Digest: "       -> s"${digest.alg}:${digest.hex}",
                "Status: "       -> (if (info.nonEmpty) "Found" else "Not Found")
              )
              _    <- info match {
                case Some(blueprint) => logBlueprint(blueprint)
                case None            => ZIO.unit
              }
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

  type Env = Logger with GraphQLGenerator with RemoteExecutor with ConfigFileReader with FileIO with SchemaRegistry

  def live: ZLayer[Env, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)
}
