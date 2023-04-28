package tailcall.runtime.model

import tailcall.runtime.model.Path.Segment
import zio.Chunk
import zio.json.JsonCodec
import zio.parser._
import zio.schema.DynamicValue

final case class Path(segments: List[Path.Segment]) {
  self =>
  def encode: Either[String, String] = Path.encode(self)

  def evaluate(input: DynamicValue): Path = {
    transform {
      case segment @ Segment.Param(mustache) => mustache.evaluate(input) match {
          case Some(value) => Segment.Literal(value)
          case None        => segment
        }
      case segment                           => segment
    }
  }

  def transform(f: Path.Segment => Path.Segment): Path = Path(segments.map(f))

  def unsafeEvaluate(input: DynamicValue): String =
    transform {
      case Path.Segment.Literal(value)  => Path.Segment.Literal(value)
      case Path.Segment.Param(mustache) => Path.Segment
          .Literal(mustache.evaluate(input).getOrElse(throw new RuntimeException("Mustache evaluation failed")))
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
    final case class Literal(value: String)    extends Segment
    final case class Param(mustache: Mustache) extends Segment
    object Param {
      def apply(value: String): Param = Param(Mustache(value))
    }
  }

  object syntax {
    val segment = (Syntax.alphaNumeric | Syntax.charIn('-', '_')).repeat
      .transform[String](_.asString, Chunk.fromIterable(_))

    val param = Mustache.syntax.transform[Segment.Param](Segment.Param(_), _.mustache)

    val literal = segment.transform[Segment.Literal](Segment.Literal(_), _.value)

    val segmentChunk = (Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])).repeat

    val route = segmentChunk.transform[Path](chunk => Path(chunk.toList), route => Chunk.from(route.segments))

  }

  object unsafe {
    def fromString(string: String): Path =
      decode(string).getOrElse(throw new RuntimeException(s"Invalid Route: ${string}"))
  }
}
