package tailcall.gateway.ast

import tailcall.gateway.internal.DynamicValueExtension._
import zio.Chunk
import zio.parser.Syntax
import zio.schema.DynamicValue

final case class Placeholder(path: Chunk[String]) {
  self =>
  def evaluate(dv: DynamicValue): Option[DynamicValue] = Placeholder.evaluate(dv, self)
}

object Placeholder {
  lazy val path = Syntax
    .alphaNumeric
    .repeat
    .transform[String](_.asString, Chunk.fromIterable(_))
    .repeatWithSep(Syntax.char('.'))
    .transform[Placeholder](Placeholder(_), _.path)

  lazy val syntax = Syntax.string("${", ()) ~ path ~ Syntax.char('}')

  def decode(string: String): Either[String, Placeholder] =
    syntax.parseString(string) match {
      case Left(_)      => Left(s"Invalid placeholder: ${string}")
      case Right(value) => Right(value)
    }

  def encode(placeholder: Placeholder): Either[String, String] =
    syntax.asPrinter.printString(placeholder)

  def evaluate(dv: DynamicValue, ph: Placeholder): Option[DynamicValue] = dv.getPath(ph.path.toList)
}
