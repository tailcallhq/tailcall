package tailcall.gateway.ast

import zio.Chunk
import zio.parser.Syntax

case class Placeholder(path: Chunk[String])

object Placeholder {
  lazy val syntax = Syntax
    .alphaNumeric
    .repeat
    .transform[String](_.asString, Chunk.fromIterable(_))
    .repeatWithSep(Syntax.char('.'))
    .transform[Placeholder](Placeholder(_), _.path)

  def decode(string: String): Either[String, Placeholder] =
    syntax.parseString(string) match {
      case Left(_)      => Left(s"Invalid placeholder: ${string}")
      case Right(value) => Right(value)
    }

  def encode(placeholder: Placeholder): Either[String, String] =
    syntax.asPrinter.printString(placeholder)
}
