package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.registry.SchemaRegistryClient
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.service.{ConfigFileIO, FileIO, GraphQLGenerator}
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
    configReader: ConfigFileIO,
    fileIO: FileIO,
    registry: SchemaRegistryClient
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

    // fixme: to fool the compiler
    private def getBaseURL: UIO[String] = ZIO.succeed("http://localhost:8080")

    override def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Compile(file, output) => for {
              config <- configReader.read(file.toFile)
              blueprint        = config.toBlueprint
              digest           = blueprint.digest
              fileName         = "tc-" + digest.hex + ".orc"
              outputFile: Path = output.getOrElse(file.getParent).resolve(fileName).toAbsolutePath
              _ <- fileIO.write(outputFile.toFile, blueprint.toJson, FileIO.defaultFlag.withCreate.withTruncateExisting)
              _ <- logSucceed("Compilation completed successfully.")
              _ <- logLabeled("Digest" -> s"${digest.hex}", "Generated File" -> fileName)
            } yield ()
          case CommandADT.GraphQLSchema(path)   => for {
              blueprint <- fileIO.readJson[Blueprint](path.toFile)
              _         <- logSucceed("GraphQL schema was successfully generated.")
              _         <- logBlueprint(blueprint)
            } yield ()
          case CommandADT.Deploy(path)          => for {
              blueprint <- fileIO.readJson[Blueprint](path.toFile)
              base      <- getBaseURL
              digest    <- registry.add(base, blueprint)
              _         <- logSucceed("Deployment was completed successfully.")
              _         <- logLabeled(
                "Remote Server:" -> base,
                "Digest"         -> s"${digest.hex}",
                "URL"            -> s"${base}/graphql/${digest.hex}"
              )
            } yield ()
          case CommandADT.Drop(digest)          => for {
              base <- getBaseURL
              _    <- registry.drop(base, digest)
              _    <- logSucceed(s"Blueprint with ID '$digest' was dropped successfully.")
              _    <- logLabeled("Remote Server" -> base, "Digest" -> s"${digest.hex}")
            } yield ()

          case CommandADT.GetAll(index, offset) => for {
              base       <- getBaseURL
              blueprints <- registry.list(base, index, offset)
              _          <- logSucceed("Listing all blueprints.")
              _          <- ZIO.foreachDiscard(blueprints.zipWithIndex) { case (blueprint, id) =>
                log(s"${id + 1}.\t${blueprint.digest.hex}")
              }
              _          <- logLabeled("Remote Server" -> base, "Total Count" -> s"${blueprints.length}")
            } yield ()

          case CommandADT.GetOne(digest) => for {
              base <- getBaseURL
              info <- registry.get(base, digest)
              _    <- logLabeled(
                "Remote Server" -> base,
                "Digest"        -> s"${digest.hex}",
                "Status"        -> (if (info.nonEmpty) "Found" else "Not Found")
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
      log(HelpDoc.blocks(labels.map { case (key, value) => HelpDoc.p(spans(text(key + ": "), strong(value))) }))
    }

    private def logSucceed(message: String): UIO[Unit] = log(text(message))
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Nothing, ExitCode] =
    ZIO.serviceWithZIO[CommandExecutor](_.dispatch(command))

  type Env = Logger with GraphQLGenerator with ConfigFileIO with FileIO with SchemaRegistryClient

  def live: ZLayer[Env, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)
}
