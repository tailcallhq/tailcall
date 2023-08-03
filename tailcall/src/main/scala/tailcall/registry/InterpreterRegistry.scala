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

  /**
   * Ignores the requests hash and returns the interpreter
   * generated from the configuration provided.
   */
  def file(file: File): ZLayer[ConfigFileIO with GraphQLGenerator, Throwable, InterpreterRegistry] =
    ZLayer.fromZIO(for {
      config      <- ConfigFileIO.readFile(file.toJava)
      blueprint   <- config.toBlueprint.toTask
      graphQL     <- GraphQLGenerator.toGraphQL(blueprint)
      interpreter <- graphQL.interpreter
    } yield new InterpreterRegistry {
      override def get(id: String): Task[Option[Interpreter]] = ZIO.succeed(Some(interpreter))
    })

  def get(hex: String): ZIO[InterpreterRegistry, Throwable, Option[Interpreter]] = ZIO.serviceWithZIO(_.get(hex))

  def live: ZLayer[SchemaRegistry with GraphQLGenerator, Nothing, InterpreterRegistry] =
    ZLayer.fromZIO(for {
      reg   <- ZIO.service[SchemaRegistry]
      gql   <- ZIO.service[GraphQLGenerator]
      cache <- Ref.make(Map.empty[String, Interpreter])
    } yield new Live(reg, gql, cache))

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
}
