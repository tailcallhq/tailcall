package tailcall.runtime.transcoder

trait TranscoderSyntax {
  implicit final class Syntax[A](private val a: A) {
    def transcode[B](implicit transcoder: Transcoder[A, Nothing, B]): B                   = transcoder.run(a).get
    def transcodeOrFailWith[B, E](implicit transcoder: Transcoder[A, E, B]): Either[E, B] = transcoder.run(a).toEither
  }
}
