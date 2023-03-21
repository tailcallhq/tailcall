package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.cli.CommandADT.Remote
import tailcall.registry.SchemaRegistryClient
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.service.{ConfigFileIO, FileIO, GraphQLGenerator}
import tailcall.runtime.transcoder.Transcoder
import zio.cli.HelpDoc
import zio.cli.HelpDoc.Span.{spans, strong, text}
import zio.{Duration, ExitCode, UIO, ZIO, ZLayer}

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

    override def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Check(file, remote)   => for {
              config <- configReader.read(file.toFile)
              blueprint = config.toBlueprint
              digest    = blueprint.digest
              remoteStatus <- remote match {
                case Some(value) => registry.get(value, digest).map {
                    case Some(_) => Option(s"${value.encode}/graphql/${digest.hex}.")
                    case None    => Option(s"GraphQL is NOT available on ${value.encode}.")
                  }
                case None        => ZIO.succeed(None)
              }

              _ <- logSucceed("Compilation completed successfully.")
              _ <- logLabeled("Digest" -> s"${digest.hex}", "Remote Schema" -> remoteStatus.getOrElse("Not Specified"))
            } yield ()
          case CommandADT.Remote(base, command) => command match {
              case Remote.Publish(path) => for {
                  config    <- configReader.read(path.toFile)
                  blueprint <- Transcoder.toBlueprint(config).toZIO
                  digest    <- registry.add(base, blueprint)
                  _         <- logSucceed("Deployment was completed successfully.")
                  _         <- logLabeled(
                    "Digest"         -> s"${digest.hex}",
                    "Remote Schema:" -> s"${base.encode}/graphql/${digest.hex}"
                  )
                } yield ()
              case Remote.Drop(digest)  => for {
                  _ <- registry.drop(base, digest)
                  _ <- logSucceed(s"Blueprint with ID '$digest' was dropped successfully.")
                  _ <- logLabeled("Remote Server" -> base.encode, "Digest" -> s"${digest.hex}")
                } yield ()

              case Remote.ShowAll(index, offset) => for {
                  blueprints <- registry.list(base, index, offset)
                  _          <- logSucceed("Listing all blueprints.")
                  _          <- ZIO.foreachDiscard(blueprints.zipWithIndex) { case (blueprint, id) =>
                    log(s"${id + 1}.\t${blueprint.digest.hex}")
                  }
                  _          <- logLabeled("Remote Server" -> base.encode, "Total Count" -> s"${blueprints.length}")
                } yield ()

              case Remote.ShowOne(digest, showSchema) => for {
                  maybe <- registry.get(base, digest)
                  _     <- logLabeled(
                    "Remote Server" -> base.encode,
                    "Digest"        -> s"${digest.hex}",
                    "Status"        -> (if (maybe.nonEmpty) "Found" else "Not Found")
                  )
                  _     <- maybe match {
                    case Some(blueprint) if showSchema => logBlueprint(blueprint)
                    case _                             => ZIO.unit
                  }
                } yield ()
            }

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

  def live: ZLayer[Env, Nothing, CommandExecutor]      = ZLayer.fromFunction(Live.apply _)
  def default: ZLayer[Any, Throwable, CommandExecutor] =
    (Logger.live ++ GraphQLGenerator.default ++ ConfigFileIO.default ++ FileIO.default ++ SchemaRegistryClient
      .default) >>> live
}
