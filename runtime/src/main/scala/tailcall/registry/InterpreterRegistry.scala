package tailcall.registry

import better.files.File
import caliban.{CalibanError, GraphQLInterpreter}
import tailcall.runtime.service.{ConfigFileIO, GraphQLGenerator, HttpContext}
import zio.{Ref, Task, ZIO, ZLayer}

trait InterpreterRegistry {
  def get(id: String): Task[Option[InterpreterRegistry.Interpreter]]
}

object InterpreterRegistry {
  private type Interpreter = GraphQLInterpreter[HttpContext, CalibanError]

  def get(hex: String): ZIO[InterpreterRegistry, Throwable, Option[Interpreter]] = ZIO.serviceWithZIO(_.get(hex))

  def live: ZLayer[SchemaRegistry with GraphQLGenerator, Nothing, InterpreterRegistry] =
    ZLayer.fromZIO(for {
      reg   <- ZIO.service[SchemaRegistry]
      gql   <- ZIO.service[GraphQLGenerator]
      cache <- Ref.make(Map.empty[String, Interpreter])
    } yield new Live(reg, gql, cache))

  /**
   * Ignores the requests hash and returns the interpreter
   * generated from the configuration provided.
   */
  def file(file: File): ZLayer[ConfigFileIO with GraphQLGenerator, Nothing, InterpreterRegistry] =
    ZLayer.fromZIO(for {
      gql   <- ZIO.service[GraphQLGenerator]
      cache <- Ref.make(Option.empty[Interpreter])
      io    <- ZIO.service[ConfigFileIO]
    } yield new Static(file, cache, io, gql))

  final private class Live(reg: SchemaRegistry, gql: GraphQLGenerator, cache: Ref[Map[String, Interpreter]])
      extends InterpreterRegistry {
    def get(hex: String): Task[Option[Interpreter]] =
      for {
        option <- cache.get.map(_.get(hex))
        int    <- option match {
          case None => for {
              option <- reg.get(hex)
              option <- option match {
                case None     => ZIO.succeed(None)
                case Some(bp) => for {
                    int <- gql.toGraphQL(bp).interpreter
                    _   <- cache.update(_ + (hex -> int))
                  } yield Some(int)
              }
            } yield option
          case int  => ZIO.succeed(int)
        }
      } yield int
  }

  final private class Static(file: File, cache: Ref[Option[Interpreter]], io: ConfigFileIO, gql: GraphQLGenerator)
      extends InterpreterRegistry {
    def get(hex: String): Task[Option[Interpreter]] =
      for {
        option <- cache.get
        int    <- option match {
          case None => load.flatMap(int => cache.set(Option(int)).as(Option(int)))
          case int  => ZIO.succeed(int)
        }
      } yield int

    private def load: ZIO[Any, Throwable, GraphQLInterpreter[HttpContext, CalibanError]] =
      for {
        config      <- io.read(file.toJava)
        blueprint   <- config.toBlueprint.toTask
        interpreter <- gql.toGraphQL(blueprint).interpreter
      } yield interpreter
  }
}
