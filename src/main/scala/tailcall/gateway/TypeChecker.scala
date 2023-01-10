package tailcall.gateway

import caliban.parsing.adt.Document
import tailcall.gateway.adt.Config

object TypeChecker {
  final case class Source(config: Config, document: Document)

  /**
   * A specialized checker to validate the config against
   * the provided schema
   */
  private def hasType(name: String) = {
    for {
      user   <- Validation.access[Source]
      result <-
        user.config.graphQL.connections.get(name) match {
          case Some(connection) =>
            Validation.value(connection)
          case None             =>
            Validation.trace(s"Missing type: $name")
        }
    } yield result
  }

  private val defaultChecker = hasType("Query")

  def check(config: Config, document: Document): List[String] =
    defaultChecker.validate(Source(config, document)).traces
}
