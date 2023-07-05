package tailcall.registry

import caliban.{CalibanError, GraphQLInterpreter}
import tailcall.runtime.service.{GraphQLGenerator, HttpContext}
import zio.{Ref, Task, ZIO, ZLayer}

trait InterpreterRegistry {
  def get(id: String): Task[Option[InterpreterRegistry.Interpreter]]
}

object InterpreterRegistry {
  type Interpreter = GraphQLInterpreter[HttpContext, CalibanError]

  def live: ZLayer[SchemaRegistry with GraphQLGenerator, Nothing, Live] =
    ZLayer.fromZIO(for {
      reg   <- ZIO.service[SchemaRegistry]
      gql   <- ZIO.service[GraphQLGenerator]
      cache <- Ref.make(Map.empty[String, Interpreter])
    } yield new Live(reg, gql, cache))

  def get(hex: String): ZIO[InterpreterRegistry, Throwable, Option[Interpreter]] = ZIO.serviceWithZIO(_.get(hex))

  final class Live(reg: SchemaRegistry, gql: GraphQLGenerator, cache: Ref[Map[String, Interpreter]])
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
