package tailcall.runtime.ast

import tailcall.runtime.internal.DynamicValueUtil.{asString, getPath}
import zio.Chunk
import zio.parser._
import zio.schema.DynamicValue

/**
 * Custom implementation of mustache syntax
 */
final case class Mustache(path: Chunk[String]) {
  self =>
  def evaluate(input: DynamicValue): Option[String] = getPath(input, self.path.toList).flatMap(asString(_))
}

object Mustache {
  def apply(path: String*): Mustache = Mustache(Chunk.fromIterable(path))

  lazy val syntax: Syntax[String, Char, Char, Mustache] = Syntax.string("{{", ()) ~ Syntax.alphaNumeric.repeat
    .transform[String](_.asString, Chunk.fromIterable(_)).repeatWithSep(Syntax.char('.'))
    .transform[Mustache](Mustache(_), _.path) ~ Syntax.string("}}", ())

  def evaluate(string: String, input: DynamicValue): String =
    syntax.parseString(string) match {
      case Left(_)         => string
      case Right(mustache) => mustache.evaluate(input).getOrElse(string)
    }
}
