package tailcall.runtime.model

import tailcall.runtime.internal.DynamicValueUtil.{asString, getPath}
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Mustache.MustacheExpression
import zio.Chunk
import zio.parser._
import zio.schema.DynamicValue

/**
 * Custom implementation of mustache syntax
 */
final case class Mustache(tokens: Chunk[Mustache.Token]) {
  def evaluate(input: DynamicValue): Mustache =
    Mustache {
      tokens.map {
        case token @ Mustache.TextNode(_)           => token
        case token @ Mustache.MustacheExpression(_) => MustacheExpression.evaluate(token, input)
            .map(Mustache.TextNode(_)).getOrElse(token)
      }
    }

  def isLiteral: Boolean = tokens.forall(_.isInstanceOf[Mustache.TextNode])
}

object Mustache {
  lazy val syntax: Syntax[String, Char, Char, Mustache] =
    (TextNode.syntax.widen[Token] | MustacheExpression.syntax.widen[Token]).repeat.transform(Mustache(_), _.tokens)

  def apply(tokens: Mustache.Token*): Mustache = Mustache(Chunk.fromIterable(tokens))

  def prm(path: String*): Token = MustacheExpression(path: _*)

  def txt(value: String): Token = TextNode(value)

  sealed trait Token
  final case class TextNode(value: String)                 extends Token
  final case class MustacheExpression(path: Chunk[String]) extends Token

  object TextNode {
    lazy val syntax: Syntax[String, Char, Char, TextNode] = Syntax.charNotIn("{{}}").repeat
      .transform[TextNode](chunk => TextNode(chunk.mkString), literal => Chunk.fromIterable(literal.value))
  }

  object MustacheExpression {
    lazy val syntax: Syntax[String, Char, Char, MustacheExpression] = Syntax.string("{{", ()) ~ Syntax.alphaNumeric
      .repeat.transform[String](_.asString, Chunk.fromIterable(_)).repeatWithSep(Syntax.char('.'))
      .transform[MustacheExpression](MustacheExpression(_), _.path) ~ Syntax.string("}}", ())

    def apply(path: String*): MustacheExpression = MustacheExpression(Chunk.fromIterable(path))

    def evaluate(string: String, input: DynamicValue): TValid[String, String] =
      syntax.parseString(string) match {
        case Left(error)     => TValid.fail(s"Invalid mustache expression: ${string}: ${error.toString}")
        case Right(mustache) => MustacheExpression.evaluate(mustache, input)
      }

    def evaluate(mustache: MustacheExpression, input: DynamicValue): TValid[String, String] = {
      for {
        value  <- TValid.fromOption(getPath(input, mustache.path.toList), s"Path ${mustache.path} not found")
        string <- TValid.fromOption(asString(value), s"Value $value is not a string")
      } yield string
    }
  }
}
