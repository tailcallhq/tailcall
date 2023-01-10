package tailcall.gateway

import zio._
import tailcall.gateway.adt.Config
import caliban.parsing.adt.Document

import TypeChecker._

/**
 * Type checks the schema against the config. It takes in an
 * input config and schema and returns a list of failure
 * messages and a list of values.
 */
final case class TypeChecker[+A](validate: (Config, Document) => Result[A]) {
  self =>
  import TypeChecker._

  def ++[A1 >: A](other: TypeChecker[A1]): TypeChecker[A1] = {
    TypeChecker { (config, document) =>
      self.validate(config, document) ++ other.validate(config, document)
    }
  }

  def flatMap[B](f: A => TypeChecker[B]): TypeChecker[B] = {
    TypeChecker { (config, document) =>
      self.validate(config, document).flatMap(a => f(a).validate(config, document))
    }
  }

  def map[B](f: A => B): TypeChecker[B] = {
    self.flatMap(a => value(f(a)))
  }
}

object TypeChecker {
  def value[A](a: A): TypeChecker[A] = {
    TypeChecker((_: Config, Document) => Result.value(a))
  }

  def trace(messages: String): TypeChecker[Nothing] = {
    TypeChecker((_: Config, _: Document) => Result.trace(messages))
  }

  def config: TypeChecker[Config] = {
    TypeChecker((config: Config, _: Document) => Result.value(config))
  }

  def document: TypeChecker[Document] = {
    TypeChecker((_: Config, document: Document) => Result.value(document))
  }

  val empty: TypeChecker[Nothing] = {
    TypeChecker((_: Config, _: Document) => Result.empty)
  }

  final case class Result[+A](traces: List[String], values: List[A]) {
    self =>
    def ++[A1 >: A](other: Result[A1]): Result[A1] = {
      Result(self.traces ++ other.traces, self.values ++ other.values)
    }

    def flatMap[B](f: A => Result[B]): Result[B] = {
      val result = self.values.map(f(_))
      val traces = result.flatMap(_.traces)
      val values = result.flatMap(_.values)
      Result(self.traces ++ traces, values)
    }

    def map[B](f: A => B): Result[B] = {
      self.flatMap(a => Result.value(f(a)))
    }
  }

  object Result {
    def trace(message: String): Result[Nothing]       = trace(List(message))
    def trace(message: List[String]): Result[Nothing] = Result(message, Nil)
    def value[A](a: A): Result[A]                     = value(List(a))
    def value[A](a: List[A]): Result[A]               = Result(Nil, a)
    def empty: Result[Nothing]                        = Result(Nil, Nil)
  }

  /**
   * A specialized checker to validate the config against
   * the provided schema
   */
  private def hasType(name: String) = {
    for {
      config <- TypeChecker.config
      result <-
        config.graphQL.connections.get(name) match {
          case Some(connection) =>
            TypeChecker.value(connection)
          case None             =>
            TypeChecker.trace(s"Missing type: $name")
        }
    } yield result
  }

  private val defaultChecker = hasType("Query")

  def check(config: Config, document: Document): List[String] =
    defaultChecker.validate(config, document).traces
}
