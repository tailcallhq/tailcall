package tailcall.gateway.ast

import tailcall.gateway.internal.DynamicValueExtension._
import zio.Chunk
import zio.parser.Syntax
import zio.schema.DynamicValue

/**
 * Custom implementation of mustache syntax
 */
final case class Mustache(path: Chunk[String]):
  self =>
  def evaluate(input: DynamicValue): Option[String] = input.getPath(self.path.toList).flatMap(_.asString)

object Mustache:
  def apply(path: String*): Mustache = Mustache(Chunk.fromIterable(path))

  lazy val syntax: Syntax[String, Char, Char, Mustache] = Syntax.string("{{", ()) ~ Syntax.alphaNumeric.repeat
    .transform[String](_.asString, Chunk.fromIterable(_)).repeatWithSep(Syntax.char('.'))
    .transform[Mustache](Mustache(_), _.path) ~ Syntax.string("}}", ())

  def evaluate(string: String, input: DynamicValue): String =
    syntax.parseString(string) match
      case Left(_)         => string
      case Right(mustache) => mustache.evaluate(input).getOrElse(string)
