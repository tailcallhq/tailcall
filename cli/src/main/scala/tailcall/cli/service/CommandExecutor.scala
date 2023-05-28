package tailcall.cli.service

import caliban.GraphQL
import tailcall.cli.CommandADT
import tailcall.cli.CommandADT.{BlueprintOptions, Remote, SourceFormat, TargetFormat}
import tailcall.registry.SchemaRegistryClient
import tailcall.runtime.EndpointUnifier
import tailcall.runtime.model._
import tailcall.runtime.service._
import tailcall.runtime.transcoder.Endpoint2Config.NameGenerator
import tailcall.runtime.transcoder.Transcoder
import zio.http.URL
import zio.json.EncoderOps
import zio.{Console, Duration, ExitCode, ZIO, ZLayer}

import java.io.IOException
import java.net.ConnectException
import java.nio.file.{NoSuchFileException, Path}

trait CommandExecutor {
  def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode]
}

object CommandExecutor {
  type Env = GraphQLGenerator with ConfigFileIO with FileIO with SchemaRegistryClient with EndpointGenerator

  def default: ZLayer[Any, Throwable, CommandExecutor] = { Env.default >>> live }

  def execute(command: CommandADT): ZIO[CommandExecutor, Nothing, ExitCode] =
    ZIO.serviceWithZIO[CommandExecutor](_.dispatch(command))

  def live: ZLayer[Env, Nothing, CommandExecutor] = ZLayer.fromFunction(Live.apply _)

  private def nPlusOneData(nPlusOne: Boolean, config: Config, seq: Seq[(String, String)]) = {
    ZIO.succeed(
      if (nPlusOne)
        seq :+ ("N + 1"    -> (config.nPlusOne.length.toString + Fmt.nPlusOne(config.nPlusOne.map(_.map(_._2)))))
      else seq :+ ("N + 1" -> config.nPlusOne.length.toString)
    )
  }

  final private case class Live(
    graphQLGen: GraphQLGenerator,
    configFile: ConfigFileIO,
    fileIO: FileIO,
    registry: SchemaRegistryClient,
    endpointGen: EndpointGenerator,
  ) extends CommandExecutor {
    override def dispatch(command: CommandADT): ZIO[Any, Nothing, ExitCode] =
      timed {
        command match {
          case CommandADT.Generate(files, sourceFormat, targetFormat, write) =>
            runGenerate(files, sourceFormat, targetFormat, write)
          case CommandADT.Check(files, remote, nPlusOne, options) => runCheck(files, remote, nPlusOne, options)
          case CommandADT.Remote(base, command)                   => command match {
              case Remote.Publish(path)          => runRemotePublish(base, path)
              case Remote.Drop(digest)           => runRemoteDrop(base, digest)
              case Remote.ListAll(index, offset) => runRemoteList(base, index, offset)
              case Remote.Show(digest, options)  => runRemoteShow(base, digest, options)
            }
        }
      }.foldZIO(
        error => Console.printLine(Fmt.error(error)).as(ExitCode.failure).exitCode,
        _ => ZIO.succeed(ExitCode.success),
      )

    def timed[R, E >: IOException, A](program: ZIO[R, E, A]): ZIO[R, E, A] =
      for {
        start <- zio.Clock.nanoTime
        a     <- program
        end   <- zio.Clock.nanoTime
        _     <- Console.printLine {
          val duration = Duration.fromNanos(end - start)
          s"\n\uD83D\uDC4D Completed in ${duration.toMillis} ms."
        }
      } yield a

    def writeGeneratedFile[R, E >: Throwable](content: ZIO[R, E, String], write: Option[Path]): ZIO[R, E, Unit] =
      for {
        out <- content
        _   <- write match {
          case Some(path) => for {
              _ <- Console.printLine(Fmt.heading(s"Generated File: ${path.toString}"))
              _ <- fileIO.write(path.toFile, out)
            } yield ()
          case None       => for {
              _ <- Console.printLine(Fmt.heading("Generated Output:"))
              _ <- Console.printLine(out)
            } yield ()
        }
      } yield ()

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

    private def postman2GraphQL(files: ::[Path], dSLFormat: ConfigFormat): ZIO[Any, Throwable, String] = {
      val nameGen = NameGenerator.incremental
      for {
        postmanCollection <- ZIO.foreachPar(files.toList)(path => fileIO.readJson[Postman](path.toFile))
        endpoints         <- ZIO.foreachPar(postmanCollection)(endpointGen.generate(_)).map(_.flatten)
        mergedEndpoints   <- EndpointUnifier.unify(endpoints).toZIO
          .mapError(errors => new RuntimeException(errors.mkString(", ")))
        configs           <- ZIO.foreach(mergedEndpoints)(endpoint =>
          Transcoder.toConfig(endpoint, nameGen).toZIO.mapError(errors => new RuntimeException(errors.mkString(", ")))
        )
        out               <- dSLFormat.encode(configs.reduce(_ mergeRight _).compress)
          .catchAll(err => ZIO.fail(new RuntimeException(err)))
      } yield out
    }

    private def runCheck(
      files: ::[Path],
      remote: Option[URL],
      nPlusOne: Boolean,
      options: BlueprintOptions,
    ): ZIO[Any, Throwable, Unit] = {
      for {
        config    <- configFile.readAll(files.map(_.toFile))
        blueprint <- config.toBlueprint.toZIO.mapError(ValidationError.BlueprintGenerationError)
        digest = blueprint.digest
        seq    = Seq(
          "Digest"    -> s"${digest.hex}",
          "Endpoints" -> blueprint.endpoints.length.toString,
          "Unsafe"    -> config.unsafeSteps.length.toString,
        )
        seq <- remote match {
          case Some(remote) => registry.get(remote, digest).map {
              case Some(_) => seq :+ ("Playground", Fmt.playground(remote, digest))
              case None    => seq :+ ("Playground" -> "Unavailable")
            }
          case None         => ZIO.succeed(seq)
        }
        seq <- nPlusOneData(nPlusOne, config, seq)
        _ <- Console.printLine(Fmt.success("No errors found."))
        _ <- Console.printLine(Fmt.table(seq))
        _ <- blueprintDetails(blueprint, options)
      } yield ()
    }

    private def runGenerate(
      files: ::[Path],
      sourceFormat: SourceFormat,
      targetFormat: TargetFormat,
      write: Option[Path],
    ): ZIO[Any, Throwable, Unit] = {
      val output: ZIO[Any, Throwable, String] = (sourceFormat, targetFormat) match {
        case (SourceFormat.Postman, TargetFormat.Config(dSLFormat))          => postman2GraphQL(files, dSLFormat)
        case (SourceFormat.SchemaDefinitionLanguage, TargetFormat.JsonLines) => for {
            content   <- ZIO.foreachPar(files.toList)(path => fileIO.read(path.toFile))
            jsonLines <- ZIO.foreachPar(content)(Transcoder.toJsonLines(_))
          } yield jsonLines.mkString("\n")

        case _ => ZIO
            .fail(new RuntimeException(s"Unsupported format combination ${sourceFormat.name} to ${targetFormat.name}"))
      }
      writeGeneratedFile(output, write)
    }

    private def runRemoteDrop(base: URL, digest: Digest): ZIO[Any, Throwable, Unit] = {
      for {
        _ <- registry.drop(base, digest)
        _ <- Console.printLine(Fmt.success(s"Blueprint dropped successfully."))
        _ <- Console.printLine(Fmt.table(Seq("Digest" -> s"${digest.hex}")))
      } yield ()
    }

    private def runRemoteList(base: URL, index: Int, offset: Int): ZIO[Any, Throwable, Unit] = {
      for {
        blueprints <- registry.list(base, index, offset)
        _          <- Console.printLine(Fmt.blueprints(blueprints))
        _ <- Console.printLine(Fmt.table(Seq("Server" -> base.encode, "Total Count" -> s"${blueprints.length}")))
      } yield ()
    }

    private def runRemotePublish(base: URL, path: ::[Path]): ZIO[Any, Throwable, Unit] = {
      for {
        config    <- configFile.readAll(path.map(_.toFile))
        blueprint <- config.toBlueprint.toZIO.mapError(ValidationError.BlueprintGenerationError)
        digest    <- registry.add(base, blueprint)
        _         <- Console.printLine(Fmt.success("Deployment was completed successfully."))
        seq       <- ZIO.succeed(Seq(
          "Digest"     -> s"${digest.hex}",
          "Endpoints"  -> blueprint.endpoints.length.toString,
          "Unsafe"     -> config.unsafeSteps.length.toString,
          "Playground" -> Fmt.playground(base, digest),
        ))
        seq       <- nPlusOneData(false, config, seq)
        _         <- Console.printLine(Fmt.table(seq))
      } yield ()
    }

    private def runRemoteShow(base: URL, digest: Digest, options: BlueprintOptions): ZIO[Any, Throwable, Unit] = {
      for {
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

  object Env {
    val default: ZLayer[Any, Throwable, Env] = GraphQLGenerator.default ++ ConfigFileIO.default ++ FileIO
      .default ++ SchemaRegistryClient.default ++ EndpointGenerator.default
  }

  object Fmt {
    def blueprint(blueprint: Blueprint): String = { blueprint.toJsonPretty }

    def blueprints(blueprints: List[Blueprint]): String = {
      Fmt.table(blueprints.zipWithIndex.map { case (blueprint, index) => ((index + 1).toString, blueprint.digest.hex) })
    }

    def caption(str: String): String = fansi.Str(str).overlay(fansi.Color.DarkGray).render

    def error(message: String): String = fansi.Str(message).overlay(fansi.Color.Red).render

    def error(cause: Throwable): String =
      cause match {
        case cause: NoSuchFileException => error(s"File not found: ${cause.getMessage}")
        case cause: ConnectException    => error(s"Connection error: ${cause.getMessage}")
        case cause: ValidationError     => error(cause.getMessage())
        case cause                      => error(cause.toString)
      }

    def graphQL(graphQL: GraphQL[_]): String = { graphQL.render }

    def heading(str: String): String = fansi.Str(str).overlay(fansi.Bold.On).render

    def meta(str: String): String = fansi.Str(str).overlay(fansi.Color.LightYellow).render

    def nPlusOne(list: List[List[String]]): String =
      meta(
        list.sortBy(_.mkString("_")).map { path =>
          "  query { " + path.foldRight("") { case (value, acc) =>
            if (acc.isEmpty) value else s"${value} { ${acc} }"
          } + " }"
        }.mkString("\n", "\n", "")
      )

    def playground(url: URL, digest: Digest): String = s"${url.encode}/graphql/${digest.hex}."

    def success(str: String): String = fansi.Str(str).overlay(fansi.Color.Green).render

    def table(labels: Seq[(String, String)]): String = {
      def maxLength = labels.map(_._1.length).max + 1
      def padding   = " " * maxLength
      labels.map { case (key, value) => heading((key + ":" + padding).take(maxLength)) + " " ++ value }.mkString("\n")
    }
  }
}
