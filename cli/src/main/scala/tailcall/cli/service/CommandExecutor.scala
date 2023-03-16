package tailcall.cli.service

import tailcall.cli.CommandADT
import tailcall.runtime.ast.Blueprint
import tailcall.runtime.service.{ConfigReader, GraphQLGenerator}
import zio.cli.HelpDoc.Span.{spans, strong, text, uri}
import zio.json.{DecoderOps, EncoderOps}
import zio.{Duration, ExitCode, ZIO, ZLayer}

import java.nio.file.{Files, Path, Paths, StandardOpenOption}

trait CommandExecutor {
  def execute(command: CommandADT): ZIO[Any, Nothing, ExitCode]
}

object CommandExecutor {
  final case class Live(log: Logger, graphQL: GraphQLGenerator) extends CommandExecutor {
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

    def wrtCWD(path: Path): Path = {
      val cwd = Paths.get("").toAbsolutePath.toString
      path.toAbsolutePath.toString.stripPrefix(cwd) match {
        case "" => Paths.get(".")
        case s  => Paths.get(s)
      }
    }

    override def execute(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Remote(_, _)          => ???
          case CommandADT.Compile(file, output) => for {
              _      <- log(spans(text("Compiling: "), uri(wrtCWD(file).toUri)))
              config <- ConfigReader.config.readFile(file.toFile)
              blueprint        = config.toBlueprint
              digest           = blueprint.digest
              fileName         = "tc-" + digest.alg + "-" + digest.hex + ".orc"
              outputFile: Path = output.getOrElse(file.getParent).resolve(fileName).toAbsolutePath
              _ <- ZIO.attemptBlocking(Files.writeString(
                outputFile,
                blueprint.toJson,
                StandardOpenOption.CREATE,
                StandardOpenOption.TRUNCATE_EXISTING
              ))
              _ <- log(spans(text("Digest: "), strong(s"${digest.alg}:${digest.hex}")))
              _ <- log(spans(text("Generated File: "), strong(s"${fileName}")))
            } yield ()
          case CommandADT.GraphQLSchema(path)   => for {
              json      <- ZIO.attemptBlocking(Files.readString(path))
              blueprint <- json.fromJson[Blueprint] match {
                case Left(err)        => ZIO.fail(new RuntimeException(err))
                case Right(blueprint) => ZIO.succeed(blueprint)
              }
              _         <- log(spans(text(graphQL.toGraphQL(blueprint).render)))
            } yield ()
        }
      }.tapError(log.error(_)).exitCode
  }

  def execute(command: CommandADT): ZIO[CommandExecutor, Nothing, ExitCode] =
    ZIO.serviceWithZIO[CommandExecutor](_.execute(command))

  def live: ZLayer[Logger with GraphQLGenerator, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)
}
