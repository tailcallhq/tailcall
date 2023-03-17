package tailcall.runtime.service

import caliban.parsing.Parser
import caliban.parsing.adt.Document
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.validation.Validator
import zio.{Task, ZIO, ZLayer}

import java.io.File

trait GraphQLFileReader {
  def read(file: File): Task[Document]
}

object GraphQLFileReader {
  final case class Live(fileIO: FileIO) extends GraphQLFileReader {
    override def read(file: File): Task[Document] =
      for {
        string            <- fileIO.read(file)
        document          <- Parser.parseQuery(string)
        rootSchemaBuilder <- caliban.tools.RemoteSchema.parseRemoteSchema(document) match {
          case None           => ZIO.fail(new RuntimeException("GraphQL does not contain a schema definition"))
          case Some(__schema) => ZIO.succeed(RootSchemaBuilder(
              query = Some(Operation(__schema.queryType, Step.NullStep)),
              mutation = __schema.mutationType.map(Operation(_, Step.NullStep)),
              subscription = __schema.subscriptionType.map(Operation(_, Step.NullStep)),
              additionalTypes = __schema.types,
              schemaDirectives = Nil
            ))
        }
        _                 <- Validator.validateSchema(rootSchemaBuilder)
      } yield document
  }

  def live: ZLayer[FileIO, Nothing, GraphQLFileReader] = ZLayer.fromFunction(Live(_))
}
