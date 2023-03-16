package tailcall.runtime.service

import caliban.parsing.Parser
import caliban.parsing.adt.Document
import caliban.schema.{Operation, RootSchemaBuilder, Step}
import caliban.validation.Validator
import tailcall.runtime.dsl.json.Config
import zio.{Task, ZIO}

import java.io.File
import java.net.URL
import java.nio.file.Path
import scala.io.Source

/**
 * Reads configuration from a file.
 */
trait ConfigReader[A] {
  def readFile(file: => File): Task[A]
  final def readPath(path: => Path): Task[A] = readFile(path.toFile)
  final def readURL(url: => URL): Task[A]    = readFile(new File(url.getPath))
}

object ConfigReader {

  // TODO: replace the custom implementation with ZIO Config.
  def config: ConfigReader[Config] =
    new ConfigReader[Config] {
      override def readFile(file: => File): Task[Config] = {
        for {
          ext    <- Extension.detect(file.getName)
          string <- ZIO.attemptBlocking(Source.fromFile(file).mkString(""))
          config <- ext.decode(string)
        } yield config
      }
    }

  def document: ConfigReader[Document] =
    new ConfigReader[Document] {
      override def readFile(file: => File): Task[Document] = {
        for {
          string            <- ZIO.attemptBlocking(Source.fromFile(file).mkString(""))
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
    }
}
