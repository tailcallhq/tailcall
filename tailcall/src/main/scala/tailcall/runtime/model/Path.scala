package tailcall.runtime.model

import tailcall.runtime.internal.DynamicValueUtil
import tailcall.runtime.model.Mustache.MustacheExpression
import tailcall.runtime.model.Path.Segment
import zio.Chunk
import zio.json.JsonCodec
import zio.parser._
import zio.schema.DynamicValue

final case class Path(segments: List[Path.Segment]) {
  self =>
  def ++(other: Path): Path = Path(self.segments ++ other.segments)

  def encode: Either[String, String] = Path.encode(self)

  /**
   * Reduces the path with params to a path with literals as
   * much as possible.
   */
  def reduce(input: DynamicValue): Path = {
    transform {
      case segment @ Segment.Param(location) =>
        DynamicValueUtil.getPath(input, location).flatMap(DynamicValueUtil.asString) match {
          case Some(value) => Segment.Literal(value)
          case None        => segment
        }
      case segment                           => segment
    }
  }

  def transform(f: Path.Segment => Path.Segment): Path = Path(segments.map(f))

  /**
   * Inserts actual values in the path segments to produce a
   * url string.
   */
  def unsafeEval(input: DynamicValue): String =
    transform {
      case Path.Segment.Literal(value)  => Path.Segment.Literal(value)
      case Path.Segment.Param(location) =>
        DynamicValueUtil.getPath(input, location).flatMap(DynamicValueUtil.asString) match {
          case Some(value) => Segment.Literal(value)
          case None        => throw new RuntimeException(s"No value found for location: ${location}")
        }
    }.encode.getOrElse(throw new RuntimeException("Path encoding failed"))

  def withLiteral(literal: String): Path = Path(segments :+ Path.Segment.Literal(literal))

  def withParam(param: String): Path = Path(segments :+ Path.Segment.Param(param))
}

object Path {
  implicit lazy val routeCodec: JsonCodec[Path] = JsonCodec[String].transformOrFail(
    Path.decode,

    // TODO: handle this error more gracefully
    route => Path.encode(route).getOrElse(throw new RuntimeException("Invalid Route")),
  )

  def decode(string: String): Either[String, Path] =
    syntax.route.parseString(string) match {
      case Left(_)      => Left(s"Invalid route: ${string}")
      case Right(value) => Right(value)
    }

  def empty: Path = Path(Nil)

  def encode(route: Path): Either[String, String] = syntax.route.asPrinter.printString(route)

  sealed trait Segment

  object Segment {
    final case class Literal(value: String)        extends Segment
    final case class Param(location: List[String]) extends Segment
    object Param {
      def apply(value: String*): Param = Param(value.toList)
    }
  }

  object syntax {
    private lazy val segmentChunk = (Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])).repeat
    private lazy val segment      = (Syntax.alphaNumeric | Syntax.charIn('-', '_', '.', '~')).repeat
      .transform[String](_.asString, Chunk.fromIterable(_))
    private lazy val param        = MustacheExpression.syntax
      .transform[Segment.Param](d => Segment.Param(d.path.toList), d => MustacheExpression(Chunk.from(d.location)))
    private lazy val literal      = segment.transform[Segment.Literal](Segment.Literal(_), _.value)
    val route: Syntax[String, Char, Char, Path] = segmentChunk
      .transform[Path](chunk => Path(chunk.toList), route => Chunk.from(route.segments))
  }

  object unsafe {
    def fromString(string: String): Path =
      decode(string).getOrElse(throw new RuntimeException(s"Invalid Route: ${string}"))
  }
}
