package tailcall.runtime.model

import tailcall.runtime.internal.DynamicValueUtil.{asString, getPath}
import tailcall.runtime.internal.TValid
import zio.Chunk
import zio.json.JsonCodec
import zio.parser._
import zio.schema.DynamicValue

/**
 * Custom implementation of mustache syntax
 */
final case class Mustache(tokens: Chunk[Mustache.Token]) {
  self =>
  def encode: TValid[String, String] = TValid.fromEither(Mustache.syntax.printString(self))

  def isLiteral: Boolean = tokens.forall(_.isInstanceOf[Mustache.TextNode])
}

object Mustache {
  lazy val syntax: Syntax[String, Char, Char, Mustache] =
    (TextNode.syntax.widen[Token] | MustacheExpression.syntax.widen[Token]).repeat.transform(Mustache(_), _.tokens)

  implicit val json: JsonCodec[Mustache] = JsonCodec[String].transformOrFail[Mustache](
    string => Mustache.syntax.parseString(string).left.map(_.toString),
    mustache =>
      Mustache.syntax.printString(mustache) match {
        case Left(error)   => throw new RuntimeException(error)
        case Right(string) => string
      },
  )

  def apply(tokens: Mustache.Token*): Mustache = Mustache(Chunk.fromIterable(tokens))

  def evaluate(mustache: Mustache, input: DynamicValue): Mustache = {
    Mustache(mustache.tokens.map {
      case token @ Mustache.TextNode(_)              => token
      case token @ Mustache.MustacheExpression(path) => getPath(input, path.toList).flatMap(asString) match {
          case Some(value) => Mustache.TextNode(value)
          case None        => token
        }
    })
  }

  def evaluate(mustache: String, input: DynamicValue): TValid[String, Mustache] =
    TValid.fromEither(Mustache.syntax.parseString(mustache))
      .mapError(error => s"Invalid mustache: ${mustache}: ${error.toString}").map(Mustache.evaluate(_, input))

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
        value  <- TValid
          .fromOption(getPath(input, mustache.path.toList), s"Path ${mustache.path.mkString("[", ", ", "]")} not found")
        string <- TValid.fromOption(asString(value), s"Value $value is not a string")
      } yield string
    }
  }
}
