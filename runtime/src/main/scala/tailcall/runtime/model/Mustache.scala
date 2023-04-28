package tailcall.runtime.model

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
  lazy val syntax: Syntax[String, Char, Char, Mustache] = Syntax.string("{{", ()) ~ Syntax.alphaNumeric.repeat
    .transform[String](_.asString, Chunk.fromIterable(_)).repeatWithSep(Syntax.char('.'))
    .transform[Mustache](Mustache(_), _.path) ~ Syntax.string("}}", ())

  def apply(path: String*): Mustache = Mustache(Chunk.fromIterable(path))

  def evaluate(string: String, input: DynamicValue): String =
    syntax.parseString(string) match {
      case Left(_)         => string
      case Right(mustache) => mustache.evaluate(input).getOrElse(string)
    }

  // FIXME: rename files to match class names
  final case class Template(tokens: Chunk[Template.Token]) {
    def evaluate(input: DynamicValue): Template =
      Template {
        tokens.map {
          case token @ Template.Literal(_)          => token
          case token @ Template.Parameter(mustache) => mustache.evaluate(input) match {
              case Some(string) => Template.Literal(string)
              case None         => token
            }
        }
      }

    def isLiteral: Boolean = tokens.forall(_.isInstanceOf[Template.Literal])
  }
  object Template                                          {
    lazy val syntax: Syntax[String, Char, Char, Template] = (literalSyntax.widen[Token] | parameterSyntax.widen[Token])
      .repeat.transform(Template(_), _.tokens)

    private lazy val literalSyntax: Syntax[String, Char, Char, Literal] = Syntax.charNotIn("{{}}").repeat
      .transform[Literal](chunk => Literal(chunk.mkString), literal => Chunk.fromIterable(literal.value))

    private lazy val parameterSyntax: Syntax[String, Char, Char, Parameter] = Mustache.syntax
      .transform(Parameter(_), _.mustache)

    def apply(tokens: Template.Token*): Template = Template(Chunk.fromIterable(tokens))

    def lit(value: String): Token = Literal(value)

    def prm(path: String*): Token = Parameter(Mustache(path: _*))

    sealed trait Token

    final case class Literal(value: String) extends Token

    final case class Parameter(mustache: Mustache) extends Token
  }
}
