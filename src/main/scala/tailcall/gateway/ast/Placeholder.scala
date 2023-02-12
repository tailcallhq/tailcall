package tailcall.gateway.ast

import tailcall.gateway.internal.DynamicValueExtension._
import zio.Chunk
import zio.parser.Syntax
import zio.schema.DynamicValue

sealed trait Placeholder {
  self =>
  final def evaluate(input: DynamicValue): Option[DynamicValue] = Placeholder.evaluate(input, self)
  final def evaluateAsString(input: DynamicValue): Option[String] =
    evaluate(input).flatMap(_.asPrimitive).map(_.value.toString())
}

object Placeholder {
  final case class Cons(path: Chunk[String]) extends Placeholder
  final case class Literal(value: String)    extends Placeholder

  def apply(path: String*): Placeholder   = Cons(Chunk.fromIterable(path))
  def literal(value: String): Placeholder = Literal(value)

  lazy val consSyntax: Syntax[String, Char, Char, Cons] = Syntax.string("${", ()) ~ Syntax
    .alphaNumeric
    .repeat
    .transform[String](_.asString, Chunk.fromIterable(_))
    .repeatWithSep(Syntax.char('.'))
    .transform[Placeholder.Cons](Placeholder.Cons(_), _.path) ~ Syntax.char('}')

  lazy val literalSyntax: Syntax[Nothing, Char, Char, Literal] = Syntax
    .anyString
    .transform[Literal](Literal(_), _.value)

  lazy val syntax: Syntax[String, Char, Char, Placeholder] = consSyntax
    .widen[Placeholder] | literalSyntax.widen[Placeholder]

  def decode(string: String): Either[String, Placeholder] =
    syntax.parseString(string) match {
      case Left(_)      => Left(s"Invalid placeholder: ${string}")
      case Right(value) => Right(value)
    }

  def encode(placeholder: Placeholder): Either[String, String] =
    syntax.asPrinter.printString(placeholder)

  def evaluate(dv: DynamicValue, ph: Placeholder): Option[DynamicValue] =
    ph match {
      case Cons(path) => dv.getPath(path.toList)
      case Literal(_) => None
    }

  def evaluateOrReturn(string: String, input: DynamicValue): Either[String, DynamicValue] =
    decode(string) match {
      case Left(_)      => Left(string)
      case Right(value) => evaluate(input, value) match {
          case None        => Left(string)
          case Some(value) => Right(value)
        }
    }

  def evaluateOrReturnString(string: String, input: DynamicValue): String =
    evaluateOrReturn(string, input) match {
      case Left(value)  => value
      case Right(value) => value.asPrimitive.fold(string)(_.value.toString())
    }
}
