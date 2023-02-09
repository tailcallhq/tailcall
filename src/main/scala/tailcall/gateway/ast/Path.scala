package tailcall.gateway.ast

import zio.Chunk
import zio.json.JsonCodec
import zio.parser.Syntax

final case class Path(segments: List[Path.Segment])
object Path {
  sealed trait Segment
  object Segment {
    final case class Literal(value: String)    extends Segment
    final case class Param(value: Placeholder) extends Segment
    object Param {
      def apply(value: String): Param = Param(Placeholder(Chunk.single(value)))
    }
  }

  object syntax {
    val segment = Syntax.alphaNumeric.repeat.transform[String](_.asString, Chunk.fromIterable(_))

    val param = (Syntax.string("${", ()) ~ Placeholder.syntax ~ Syntax.char('}'))
      .transform[Segment.Param](Segment.Param(_), _.value)

    val literal = segment.transform[Segment.Literal](Segment.Literal, _.value)

    val segmentChunk = (Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])).repeat

    val route = segmentChunk
      .transform[Path](chunk => Path(chunk.toList), route => Chunk.from(route.segments))

  }

  def decode(string: String): Either[String, Path] = syntax.route.parseString(string) match {
    case Left(_)      => Left(s"Invalid route: ${string}")
    case Right(value) => Right(value)
  }

  def encode(route: Path): Either[String, String] = syntax.route.asPrinter.printString(route)

  implicit lazy val routeCodec: JsonCodec[Path] = JsonCodec[String].transformOrFail(
    Path.decode,

    // TODO: handle this error more gracefully
    route => Path.encode(route).getOrElse(throw new RuntimeException("Invalid Route"))
  )

  object unsafe {
    def fromString(string: String): Path = decode(string)
      .getOrElse(throw new RuntimeException(s"Invalid Route: ${string}"))
  }

  def empty: Path = Path(Nil)
}
