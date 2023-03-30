package tailcall.cli.service

import caliban.GraphQL
import tailcall.cli.CommandADT
import tailcall.cli.CommandADT.{BlueprintOptions, Remote}
import tailcall.registry.SchemaRegistryClient
import tailcall.runtime.ast.{Blueprint, Digest, Endpoint}
import tailcall.runtime.dsl.Postman
import tailcall.runtime.http.HttpClient
import tailcall.runtime.service.{ConfigFileIO, FileIO, GraphQLGenerator}
import tailcall.runtime.transcoder.{Postman2Endpoints, Transcoder}
import zio.http.URL
import zio.json.EncoderOps
import zio.{Console, Duration, ExitCode, ZIO, ZLayer}

import java.io.IOException

trait CommandExecutor {
  def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode]
}

object CommandExecutor {
  final case class Live(
    graphQLGen: GraphQLGenerator,
    configReader: ConfigFileIO,
    fileIO: FileIO,
    registry: SchemaRegistryClient,
  ) extends CommandExecutor {
    def timed[R, E >: IOException, A](program: ZIO[R, E, A]): ZIO[R, E, A] =
      for {
        start <- zio.Clock.nanoTime
        a     <- program.logError
        end   <- zio.Clock.nanoTime
        _     <- Console.printLine {
          val duration = Duration.fromNanos(end - start)
          s"\n\uD83D\uDC4D Completed in ${duration.toMillis} ms."
        }
      } yield a

    override def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Generate(files, sourceFormat, configFormat) => for {
              config <- sourceFormat match {
                case CommandADT.SourceFormat.POSTMAN => for {
                    postman <- ZIO.foreachPar(files.toList)(path => fileIO.readJson[Postman](path.toFile))
                    config  <- ZIO.foreachPar(postman)(
                      Transcoder.toConfig(_, Postman2Endpoints.Config(true, "https://stg.api.mosaicwellness.in"))
                        .provide(HttpClient.default)
                    )
                  } yield config.reduce(_ mergeRight _)
              }
              out    <- configFormat.encode(config)
              _      <- Console.printLine(Fmt.heading("Generated config:"))
              _      <- Console.printLine(out)
            } yield ()
          case CommandADT.Check(files, remote, options)               => for {
              config <- configReader.readAll(files.map(_.toFile))
              blueprint = config.toBlueprint
              digest    = blueprint.digest
              seq0      = Seq("Digest" -> s"${digest.hex}")
              seq1 <- remote match {
                case Some(remote) => registry.get(remote, digest).map {
                    case Some(_) => seq0 :+ ("Playground", Fmt.playground(remote, digest))
                    case None    => seq0 :+ ("Playground" -> "Unavailable")
                  }
                case None         => ZIO.succeed(seq0)
              }
              _    <- Console.printLine(Fmt.success("No errors found."))
              _    <- Console.printLine(Fmt.table(seq1))
              _    <- blueprintDetails(blueprint, options)
            } yield ()
          case CommandADT.Remote(base, command)                       => command match {
              case Remote.Publish(path) => for {
                  config    <- configReader.readAll(path.map(_.toFile))
                  blueprint <- Transcoder.toBlueprint(config).toZIO
                  digest    <- registry.add(base, blueprint)
                  _         <- Console.printLine(Fmt.success("Deployment was completed successfully."))
                  _         <- Console.printLine(
                    Fmt.table(Seq("Digest" -> s"${digest.hex}", "Playground" -> Fmt.playground(base, digest)))
                  )
                } yield ()
              case Remote.Drop(digest)  => for {
                  _ <- registry.drop(base, digest)
                  _ <- Console.printLine(Fmt.success(s"Blueprint dropped successfully."))
                  _ <- Console.printLine(Fmt.table(Seq("Digest" -> s"${digest.hex}")))
                } yield ()

              case Remote.ListAll(index, offset) => for {
                  blueprints <- registry.list(base, index, offset)
                  _          <- Console.printLine(Fmt.blueprints(blueprints))
                  _          <- Console
                    .printLine(Fmt.table(Seq("Server" -> base.encode, "Total Count" -> s"${blueprints.length}")))
                } yield ()

              case Remote.Show(digest, options) => for {
                  maybe <- registry.get(base, digest)
                  _     <- Console.printLine(Fmt.table(Seq(
                    "Digest"     -> s"${digest.hex}",
                    "Playground" -> maybe.map(_ => Fmt.playground(base, digest)).getOrElse(Fmt.meta("Unavailable")),
                  )))
                  _     <- maybe match {
                    case Some(blueprint) => blueprintDetails(blueprint, options)
                    case _               => ZIO.unit
                  }
                } yield ()
            }
        }
      }.exitCode

    private def blueprintDetails(blueprint: Blueprint, options: BlueprintOptions): ZIO[Any, IOException, Unit] = {
      for {
        _ <- Console.printLine(Fmt.heading("Blueprint:\n") ++ Fmt.blueprint(blueprint)).when(options.blueprint)
        _ <- Console.printLine(Fmt.heading("GraphQL Schema:\n") ++ Fmt.graphQL(graphQLGen.toGraphQL(blueprint)))
          .when(options.schema)
        _ <- Console.printLine(Fmt.heading("Endpoints:\n") ++ endpoints(blueprint.endpoints)).when(options.endpoints)
      } yield ()
    }

    private def endpoints(endpoints: List[Endpoint]): String =
      List[String](
        endpoints.map[String](endpoint =>
          List[String](
            "\n",
            Fmt.heading(s"${endpoint.method.name} ${endpoint.url}"),
            Fmt.heading(s"Input Schema: ") + s"${endpoint.input.fold("Any")("\n" + _.toJsonPretty)}",
            Fmt.heading(s"Output Schema: ") + s" ${endpoint.output.fold("Nothing")("\n" + _.toJsonPretty)}",
          ).mkString("\n")
        ).mkString("\n")
      ).mkString("\n")

  }
  def execute(command: CommandADT): ZIO[CommandExecutor, Nothing, ExitCode] =
    ZIO.serviceWithZIO[CommandExecutor](_.dispatch(command))

  type Env = GraphQLGenerator with ConfigFileIO with FileIO with SchemaRegistryClient

  def live: ZLayer[Env, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)

  def default: ZLayer[Any, Throwable, CommandExecutor] =
    (GraphQLGenerator.default ++ ConfigFileIO.default ++ FileIO.default ++ SchemaRegistryClient.default) >>> live

  object Fmt {
    def success(str: String): String = fansi.Str(str).overlay(fansi.Color.Green).render

    def heading(str: String): String = fansi.Str(str).overlay(fansi.Bold.On).render

    def caption(str: String): String = fansi.Str(str).overlay(fansi.Color.DarkGray).render

    def meta(str: String): String = fansi.Str(str).overlay(fansi.Color.LightYellow).render

    def graphQL(graphQL: GraphQL[_]): String = { graphQL.render }

    def blueprint(blueprint: Blueprint): String = { blueprint.toJsonPretty }

    def blueprints(blueprints: List[Blueprint]): String = {
      Fmt.table(blueprints.zipWithIndex.map { case (blueprint, index) => ((index + 1).toString, blueprint.digest.hex) })
    }

    def table(labels: Seq[(String, String)]): String = {
      def maxLength = labels.map(_._1.length).max + 1
      def padding   = " " * maxLength
      labels.map { case (key, value) => heading((key + ":" + padding).take(maxLength)) + " " ++ value }.mkString("\n")
    }

    def playground(url: URL, digest: Digest): String = s"${url.encode}/graphql/${digest.hex}."
  }
}
